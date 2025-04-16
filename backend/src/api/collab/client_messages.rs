use crate::api::collab::{
    msg_sync::{MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE, MSG_SYNC_UPDATE, sync_response},
    txn_origin::{Actor, YOrigin},
};
use crate::api::{
    collab::{
        awareness::AwarenessUpdate,
        client::{CLOSE_ERROR, CLOSE_NORMAL, ClientClosure, ClientReceiver},
        projects_state::ProjectState,
    },
    model::User,
};
use anyhow::{Result, anyhow};
use axum::extract::ws::Message;
use rand::random;
use std::{fmt, ops::ControlFlow, sync::Arc, time::Duration};
use tokio::sync::mpsc::Receiver;
use tokio::{sync::mpsc::Sender, time::timeout};
use uuid::Uuid;
use yrs::{
    StateVector, Update,
    encoding::read::Read as _,
    updates::decoder::{Decode as _, DecoderV1},
};

use super::msg_sync::{MSG_KOSO_AWARENESS, MSG_KOSO_AWARENESS_UPDATE};

/// ClientMessageReceiver receives messages from clients
/// about a particular project and forwards the binary ones to
/// process_msg_tx for handling by `ClientMessageProcessor`.
///
/// When clients disconnect, perhaps by closing their browser tab,
/// we'll recieve a Close message and remove the client.
pub(super) struct ClientMessageReceiver {
    project: Arc<ProjectState>,
    process_msg_tx: Sender<ClientMessage>,
    receiver: ClientReceiver,
}

impl ClientMessageReceiver {
    pub(super) fn new(
        project: Arc<ProjectState>,
        process_msg_tx: Sender<ClientMessage>,
        receiver: ClientReceiver,
    ) -> Self {
        ClientMessageReceiver {
            project,
            process_msg_tx,
            receiver,
        }
    }

    /// Listen for update or close messages sent by a client.
    #[tracing::instrument(skip(self), fields(?receiver=self.receiver))]
    pub(super) async fn receive_messages_from_client(mut self) {
        // Bound how long a connection can stay open for.
        // Add [0, 20] minutes of jitter to avoid a thundering heard of reconnects.
        let timeout_duration = Duration::from_secs(60 * 60)
            + Duration::from_millis((random::<f32>() * 20.0 * 60.0 * 1000.0) as u64);
        let closure = match timeout(timeout_duration, async {
            loop {
                let Some(msg) = self.receiver.next().await else {
                    return ClientClosure {
                        code: CLOSE_NORMAL,
                        reason: "Read None from client socket.",
                        details: "Read None from client socket.".to_string(),
                        client_initiated: true,
                    };
                };
                if let ControlFlow::Break(closure) = self.receive_message_from_client(msg).await {
                    return closure;
                }
            }
        })
        .await
        {
            Ok(closure) => closure,
            Err(e) => ClientClosure {
                code: CLOSE_NORMAL,
                reason: "Resetting old connection",
                details: format!("Resetting old connection after {e}"),
                client_initiated: false,
            },
        };

        self.project
            .remove_and_close_client(&self.receiver.who, closure)
            .await;
    }

    async fn receive_message_from_client(
        &self,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<ClientClosure> {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Err(e) = self
                    .process_msg_tx
                    .send(ClientMessage {
                        who: self.receiver.who.clone(),
                        user: self.receiver.user.clone(),
                        project: Arc::clone(&self.project),
                        id: Uuid::new_v4().to_string(),
                        data: data.into(),
                    })
                    .await
                {
                    tracing::error!("Error sending message to process_msg channel: {e:?}");
                };
                ControlFlow::Continue(())
            }
            Ok(Message::Close(c)) => {
                let details = if let Some(cf) = &c {
                    format!(
                        "Client closed connection: code:'{}', detail:'{}'",
                        cf.code, cf.reason
                    )
                } else {
                    "Client closed connection: code:'NONE', detail:'No CloseFrame'".to_string()
                };
                ControlFlow::Break(ClientClosure {
                    code: CLOSE_NORMAL,
                    reason: "Client closed connection.",
                    details,
                    client_initiated: true,
                })
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e:?}");
                ControlFlow::Break(ClientClosure {
                    code: CLOSE_ERROR,
                    reason: "Failed to read from client socket.",
                    details: format!("Failed to read from client socket: {e:#}"),
                    client_initiated: true,
                })
            }
            // Our clients send heartbeat pings in the form of empty text messages
            // because, unfortunately, the javascript library doesn't support Ping messages.
            Ok(Message::Text(msg)) if msg.is_empty() => ControlFlow::Continue(()),
            Ok(_) => {
                tracing::warn!("Discarding unsolicited message: {msg:?}");
                ControlFlow::Continue(())
            }
        }
    }
}

/// ClientMessageProcessor processes messages sent by ClientMessageReceiver.
/// See the `api::collab::Collab` documentation for details on the protocol.
pub(super) struct ClientMessageProcessor {
    process_msg_rx: Receiver<ClientMessage>,
}

impl ClientMessageProcessor {
    pub(super) fn new(process_msg_rx: Receiver<ClientMessage>) -> Self {
        ClientMessageProcessor { process_msg_rx }
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn process_messages(mut self) {
        loop {
            let Some(msg) = self.process_msg_rx.recv().await else {
                break;
            };
            if let Err(e) = self.process_message(msg).await {
                tracing::warn!("Failed to process message: {e:?}");
            }
        }
        tracing::info!("Stopped processing messages");
    }

    #[tracing::instrument(skip(self))]
    async fn process_message(&self, msg: ClientMessage) -> Result<()> {
        let mut decoder = DecoderV1::from(msg.data.as_slice());
        match decoder.read_var()? {
            MSG_SYNC => match decoder.read_var()? {
                MSG_SYNC_REQUEST => {
                    tracing::debug!("Handling sync_request message");
                    let update = msg
                        .project
                        .encode_state_as_update(&StateVector::decode_v1(decoder.read_buf()?)?)
                        .await?;

                    // Respond to the client with a sync_response message containing
                    // changes known to the server but not the client.
                    // There's no need to broadcast such updates to others or perist them.
                    tracing::debug!("Sending synce_response message to client.");
                    msg.project
                        .send_msg(&msg.who, sync_response(&update))
                        .await?;
                    Ok(())
                }
                MSG_SYNC_RESPONSE | MSG_SYNC_UPDATE => {
                    tracing::debug!("Handling sync_update|sync_response message");
                    let update = Update::decode_v2(decoder.read_buf()?)?;
                    msg.project
                        .apply_doc_update(
                            YOrigin {
                                who: msg.who,
                                id: msg.id,
                                actor: Actor::User(msg.user),
                            },
                            update,
                        )
                        .await?;
                    Ok(())
                }
                invalid_type => Err(anyhow!("Invalid sync type: {invalid_type}")),
            },
            MSG_KOSO_AWARENESS => match decoder.read_var()? {
                MSG_KOSO_AWARENESS_UPDATE => {
                    if msg.user.email == "leonhard.kyle@gmail.com" {
                        panic!("Panickking now!");
                    }
                    let update: AwarenessUpdate = serde_json::from_str(decoder.read_string()?)?;
                    tracing::debug!("{update:?}");
                    msg.project
                        .update_awareness(&msg.who, &msg.user, update)
                        .await;
                    Ok(())
                }
                invalid_type => Err(anyhow!("Invalid Koso awareness type: {invalid_type}")),
            },
            invalid_type => Err(anyhow!("Invalid message protocol type: {invalid_type}")),
        }
    }
}

pub(super) struct ClientMessage {
    pub(super) who: String,
    pub(super) user: User,
    pub(super) project: Arc<ProjectState>,
    /// Unique ID associated with this update.
    pub(super) id: String,
    /// Binary contents of the client message.
    pub(super) data: Vec<u8>,
}

impl fmt::Debug for ClientMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YrsMessage")
            .field("project_id", &self.project.project_id)
            .field("who", &self.who)
            .field("id", &self.id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}

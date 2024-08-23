use crate::api::collab::{
    client::{ClientClosure, ClientReceiver, CLOSE_ERROR, CLOSE_NORMAL},
    projects_state::ProjectState,
};
use axum::extract::ws::Message;
use rand::random;
use std::{fmt, ops::ControlFlow, sync::Arc, time::Duration};
use tokio::{sync::mpsc::Sender, time::timeout};
use uuid::Uuid;

/// ClientMessageHandler receives messages from clients
/// about a particular project and forwards the binary ones to
/// process_msg_tx for handling by `YrsMessageProcessor`.
///
/// When clients disconnect, perhaps by closing their browser tab,
/// we'll recieve a Close message and remove the client.
pub(super) struct ClientMessageHandler {
    project: Arc<ProjectState>,
    process_msg_tx: Sender<YrsMessage>,
    receiver: ClientReceiver,
}

impl ClientMessageHandler {
    pub(super) fn new(
        project: Arc<ProjectState>,
        process_msg_tx: Sender<YrsMessage>,
        receiver: ClientReceiver,
    ) -> Self {
        ClientMessageHandler {
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
                let msg = self.receiver.next().await?;
                if let ControlFlow::Break(closure) = self.receive_message_from_client(msg).await {
                    return Some(closure);
                }
            }
        })
        .await
        {
            Ok(closure) => closure,
            Err(e) => Some(ClientClosure {
                code: CLOSE_NORMAL,
                reason: "Resetting old connection",
                details: format!("Resetting old connection after {e}"),
            }),
        };

        tracing::debug!("Stopped receiving messages from client");
        if let Some(closure) = closure {
            self.project
                .remove_and_close_client(&self.receiver.who, closure)
                .await;
        }
    }

    async fn receive_message_from_client(
        &self,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<ClientClosure> {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Err(e) = self
                    .process_msg_tx
                    .send(YrsMessage {
                        who: self.receiver.who.clone(),
                        project: Arc::clone(&self.project),
                        id: Uuid::new_v4().to_string(),
                        data,
                    })
                    .await
                {
                    tracing::error!("Error sending message to process_msg channel: {e}");
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
                })
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                ControlFlow::Break(ClientClosure {
                    code: CLOSE_ERROR,
                    reason: "Failed to read from client socket.",
                    details: format!("Failed to read from client socket: {e}"),
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

pub(super) struct YrsMessage {
    pub(super) who: String,
    pub(super) project: Arc<ProjectState>,
    pub(super) id: String,
    pub(super) data: Vec<u8>,
}

impl fmt::Debug for YrsMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YrsMessage")
            .field("project_id", &self.project.project_id)
            .field("who", &self.who)
            .field("id", &self.id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}

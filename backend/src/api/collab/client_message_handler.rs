use super::{
    client::{ClientClosure, ClientReceiver, CLOSE_NORMAL},
    projects_state::ProjectState,
};
use crate::api::collab::client::CLOSE_ERROR;
use axum::extract::ws::Message;
use std::{fmt, ops::ControlFlow, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct ClientMessageHandler {
    pub project: Arc<ProjectState>,
    pub process_tx: Sender<YrsMessage>,
    pub cancel: CancellationToken,
    pub receiver: ClientReceiver,
}

impl ClientMessageHandler {
    /// Listen for update or close messages sent by a client.
    #[tracing::instrument(skip(self), fields(?receiver=self.receiver))]
    pub async fn receive_messages_from_client(mut self) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = self.receiver.next() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    if let ControlFlow::Break(_) = self.receive_message_from_client(msg).await {
                        break;
                    }
                }
            }
        }
        tracing::debug!("Stopped receiving messages from client");
    }

    async fn receive_message_from_client(
        &self,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<()> {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Err(e) = self
                    .process_tx
                    .send(YrsMessage {
                        who: self.receiver.who.clone(),
                        project: Arc::clone(&self.project),
                        id: Uuid::new_v4().to_string(),
                        data,
                    })
                    .await
                {
                    tracing::error!("Error sending message to process channel: {e}");
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
                self.project
                    .remove_and_close_client(
                        &self.receiver.who,
                        ClientClosure {
                            code: CLOSE_NORMAL,
                            reason: "Client closed connection.",
                            details,
                        },
                    )
                    .await;
                ControlFlow::Break(())
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                self.project
                    .remove_and_close_client(
                        &self.receiver.who,
                        ClientClosure {
                            code: CLOSE_ERROR,
                            reason: "Failed to read from client socket.",
                            details: format!("Failed to read from client socket: {e}"),
                        },
                    )
                    .await;
                ControlFlow::Break(())
            }
            Ok(_) => {
                tracing::warn!("Discarding unsolicited message: {msg:?}");
                ControlFlow::Continue(())
            }
        }
    }
}

pub struct YrsMessage {
    pub who: String,
    pub project: Arc<ProjectState>,
    pub id: String,
    pub data: Vec<u8>,
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

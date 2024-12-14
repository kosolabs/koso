use super::User;
use crate::api::model::ProjectId;
use axum::extract::ws::{CloseCode, CloseFrame, Message, WebSocket};
use futures::SinkExt as _;
use std::fmt;

/// Splits a socket into a read, ClientReceiver, and write, ClientSender, side.
pub(super) fn from_socket(
    socket: WebSocket,
    who: &str,
    user: &User,
    project_id: &ProjectId,
) -> (ClientSender, ClientReceiver) {
    use futures::stream::StreamExt;
    let (ws_sender, ws_receiver) = socket.split();
    (
        ClientSender {
            ws_sender,
            who: who.to_owned(),
            user: user.clone(),
            project_id: project_id.clone(),
        },
        ClientReceiver {
            ws_receiver,
            who: who.to_owned(),
            project_id: project_id.clone(),
        },
    )
}

// https://www.rfc-editor.org/rfc/rfc6455.html#section-7.4.1
// https://www.iana.org/assignments/websocket/websocket.xhtml#close-code-number
pub(super) const CLOSE_NORMAL: u16 = 1000;
pub(super) const CLOSE_ERROR: u16 = 1011;
pub(super) const CLOSE_RESTART: u16 = 1012;
pub(super) const OVERLOADED: u16 = 1013;
pub(super) const CLOSE_UNAUTHORIZED: u16 = 3000;

pub(super) struct ClientClosure {
    pub(super) code: CloseCode,
    /// Reason sent to the client.
    /// Must not contain anything sensitive.
    pub(super) reason: &'static str,
    /// Additional details for internal logging.
    pub(super) details: String,
}

pub(super) struct ClientSender {
    ws_sender: futures::stream::SplitSink<WebSocket, Message>,
    pub(super) who: String,
    pub(super) user: User,
    pub(super) project_id: ProjectId,
}

impl ClientSender {
    pub(super) async fn send(&mut self, data: Vec<u8>) -> Result<(), axum::Error> {
        self.ws_sender.send(Message::Binary(data)).await
    }

    pub(super) async fn close(&mut self, code: CloseCode, reason: &'static str) {
        let _ = self
            .ws_sender
            .send(Message::Close(Some(CloseFrame {
                code,
                reason: reason.into(),
            })))
            .await;
        let _ = self.ws_sender.close().await;
    }
}

impl fmt::Debug for ClientSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientSender")
            .field("who", &self.who)
            .field("project_id", &self.project_id)
            .finish()
    }
}

pub(super) struct ClientReceiver {
    ws_receiver: futures::stream::SplitStream<WebSocket>,
    pub(super) who: String,
    pub(super) project_id: ProjectId,
}

impl ClientReceiver {
    pub(super) async fn next(&mut self) -> Option<Result<Message, axum::Error>> {
        use futures::stream::StreamExt;
        self.ws_receiver.next().await
    }
}

impl fmt::Debug for ClientReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientReceiver")
            .field("who", &self.who)
            .field("project_id", &self.project_id)
            .finish()
    }
}

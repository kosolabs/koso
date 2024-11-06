use crate::api::collab::{
    client_message_handler::ClientMessage,
    msg_sync::{sync_response, MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE, MSG_SYNC_UPDATE},
    txn_origin::YOrigin,
};
use anyhow::{anyhow, Result};
use tokio::sync::mpsc::Receiver;
use yrs::{
    encoding::read::Read as _,
    updates::decoder::{Decode as _, DecoderV1},
    StateVector, Update,
};

/// YrsMessageProcessor processes messages sent by ClientMessageHandlers.
/// See the `api::collab::Collab` documentation for details on the protocol.
pub(super) struct YrsMessageProcessor {
    process_msg_rx: Receiver<ClientMessage>,
}

impl YrsMessageProcessor {
    pub(super) fn new(process_msg_rx: Receiver<ClientMessage>) -> Self {
        YrsMessageProcessor { process_msg_rx }
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn process_messages(mut self) {
        loop {
            let Some(msg) = self.process_msg_rx.recv().await else {
                break;
            };
            if let Err(e) = self.process_message(msg).await {
                tracing::warn!("Failed to process message: {e}");
            }
        }
        tracing::info!("Stopped processing messages");
    }

    #[tracing::instrument(skip(self))]
    async fn process_message(&self, msg: ClientMessage) -> Result<()> {
        let mut decoder = DecoderV1::from(msg.data.as_slice());
        match decoder.read_var()? {
            MSG_SYNC => {
                match decoder.read_var()? {
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
                                },
                                update,
                            )
                            .await?;

                        Ok(())
                    }
                    invalid_type => Err(anyhow!("Invalid sync type: {invalid_type}")),
                }
            }
            invalid_type => Err(anyhow!("Invalid message protocol type: {invalid_type}")),
        }
    }
}

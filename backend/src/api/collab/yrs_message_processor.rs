use super::client_message_handler::YrsMessage;
use crate::api::collab::{
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

pub struct YrsMessageProcessor {
    pub process_rx: Receiver<YrsMessage>,
}

impl YrsMessageProcessor {
    #[tracing::instrument(skip(self))]
    pub async fn process_messages(mut self) {
        loop {
            let msg = self.process_rx.recv().await;
            let Some(msg) = msg else {
                break;
            };
            if let Err(e) = self.process_message(msg).await {
                tracing::warn!("Failed to process message: {e}");
            }
        }
        tracing::info!("Stopped processing messages");
    }

    #[tracing::instrument(skip(self))]
    async fn process_message(&self, msg: YrsMessage) -> Result<()> {
        let mut decoder = DecoderV1::from(msg.data.as_slice());
        match decoder.read_var()? {
            MSG_SYNC => {
                match decoder.read_var()? {
                    MSG_SYNC_REQUEST => {
                        tracing::debug!("Handling sync_request message");
                        let update = {
                            let sv: StateVector = StateVector::decode_v1(decoder.read_buf()?)?;
                            msg.project.encode_state_as_update(&sv).await?
                        };

                        // Respond to the client with a sync_response message containing
                        // changes known to the server but not the client.
                        // There's no need to broadcast such updates to others or perist them.
                        tracing::debug!("Sending synce_response message to client.");
                        msg.project
                            .send_msg(&msg.who, sync_response(update))
                            .await?;

                        Ok(())
                    }
                    MSG_SYNC_RESPONSE | MSG_SYNC_UPDATE => {
                        tracing::debug!("Handling sync_update|sync_response message");
                        let update = decoder.read_buf()?.to_vec();
                        {
                            let update = Update::decode_v2(&update)?;
                            msg.project
                                .apply_doc_update(
                                    YOrigin {
                                        who: msg.who.clone(),
                                        id: msg.id.clone(),
                                    },
                                    update,
                                )
                                .await?;
                        }

                        Ok(())
                    }
                    invalid_type => Err(anyhow!("Invalid sync type: {invalid_type}")),
                }
            }
            invalid_type => Err(anyhow!("Invalid message protocol type: {invalid_type}")),
        }
    }
}

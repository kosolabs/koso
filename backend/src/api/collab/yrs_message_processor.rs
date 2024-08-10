use super::client_message_handler::YrsMessage;
use crate::api::collab::{
    msg_sync::{sync_response, MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE, MSG_SYNC_UPDATE},
    projects_state::DocBox,
    txn_origin::as_origin,
};
use anyhow::{anyhow, Result};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use yrs::{
    encoding::read::Read as _,
    updates::decoder::{Decode as _, DecoderV1},
    ReadTxn as _, StateVector, Transact as _, Update,
};

pub struct YrsMessageProcessor {
    pub process_rx: Receiver<YrsMessage>,
    pub cancel: CancellationToken,
}

impl YrsMessageProcessor {
    #[tracing::instrument(skip(self))]
    pub async fn process_messages(mut self) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = self.process_rx.recv() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    self.process_message(msg).await;
                }
            }
        }
        tracing::info!("Stopped processing messages");
    }

    #[tracing::instrument(skip(self))]
    async fn process_message(&self, msg: YrsMessage) {
        if let Err(e) = self.process_message_internal(msg).await {
            tracing::warn!("Failed to process message: {e}");
        }
    }

    async fn process_message_internal(&self, msg: YrsMessage) -> Result<()> {
        let mut decoder = DecoderV1::from(msg.data.as_slice());
        match decoder.read_var()? {
            MSG_SYNC => {
                match decoder.read_var()? {
                    MSG_SYNC_REQUEST => {
                        tracing::debug!("Handling sync_request message");
                        let update = {
                            let sv: StateVector = StateVector::decode_v1(decoder.read_buf()?)?;
                            DocBox::doc_or_error(msg.project.doc_box.lock().await.as_ref())?
                                .doc
                                .transact()
                                .encode_state_as_update_v2(&sv)
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
                            DocBox::doc_or_error(msg.project.doc_box.lock().await.as_ref())?
                                .doc
                                .transact_mut_with(as_origin(&msg.who, &msg.id))
                                .apply_update(update);
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

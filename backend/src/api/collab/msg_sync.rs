use yrs::{
    StateVector,
    encoding::write::Write as _,
    updates::encoder::{Encode as _, Encoder as _, EncoderV1},
};

pub(crate) const MSG_SYNC: u8 = 0;

pub(crate) const MSG_SYNC_REQUEST: u8 = 0;
pub(crate) const MSG_SYNC_RESPONSE: u8 = 1;
pub(crate) const MSG_SYNC_UPDATE: u8 = 2;

pub(crate) const MSG_KOSO_AWARENESS: u8 = 8;

pub(crate) const MSG_KOSO_AWARENESS_UPDATE: u8 = 0;
pub(crate) const MSG_KOSO_AWARENESS_STATE: u8 = 1;

pub(crate) fn sync_request(sv: &StateVector) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_SYNC);
    encoder.write_var(MSG_SYNC_REQUEST);
    encoder.write_buf(sv.encode_v1());
    encoder.to_vec()
}

pub(crate) fn sync_response(update: &[u8]) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_SYNC);
    encoder.write_var(MSG_SYNC_RESPONSE);
    encoder.write_buf(update);
    encoder.to_vec()
}

pub(crate) fn sync_update(update: &[u8]) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_SYNC);
    encoder.write_var(MSG_SYNC_UPDATE);
    encoder.write_buf(update);
    encoder.to_vec()
}

pub(crate) fn koso_awareness_state(state: &str) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_KOSO_AWARENESS);
    encoder.write_var(MSG_KOSO_AWARENESS_STATE);
    encoder.write_string(state);
    encoder.to_vec()
}

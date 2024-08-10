use yrs::{
    encoding::write::Write as _,
    updates::encoder::{Encode as _, Encoder as _, EncoderV1},
    StateVector,
};

pub const MSG_SYNC: u8 = 0;

pub const MSG_SYNC_REQUEST: u8 = 0;
pub const MSG_SYNC_RESPONSE: u8 = 1;
pub const MSG_SYNC_UPDATE: u8 = 2;

pub fn sync_request(sv: StateVector) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_SYNC);
    encoder.write_var(MSG_SYNC_REQUEST);
    encoder.write_buf(sv.encode_v1());
    encoder.to_vec()
}

pub fn sync_response(update: Vec<u8>) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_SYNC);
    encoder.write_var(MSG_SYNC_RESPONSE);
    encoder.write_buf(update);
    encoder.to_vec()
}

pub fn sync_update(update: Vec<u8>) -> Vec<u8> {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MSG_SYNC);
    encoder.write_var(MSG_SYNC_UPDATE);
    encoder.write_buf(update);
    encoder.to_vec()
}

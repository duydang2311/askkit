use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::Decoder,
};

use crate::common::error::AppError;

pub struct SseDecoder;

impl SseDecoder {
    pub fn new() -> Self {
        SseDecoder
    }
}

impl Decoder for SseDecoder {
    type Item = String;
    type Error = AppError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<String>, Self::Error> {
        if let Some(i) = buf.windows(2).position(|window| window == b"\n\n") {
            let line = buf.split_to(i);
            buf.advance(2);
            let s = String::from_utf8(line.to_vec()).map_err(Self::Error::from)?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }
}

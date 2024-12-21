use std::{io, marker::PhantomData, num::NonZero};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio_util::{
    bytes::{Buf, BufMut, BytesMut},
    codec::{Decoder, Encoder},
};

const SERDE_BRIEF_CFG: serde_brief::Config = serde_brief::Config {
    max_size: NonZero::new(16 * (2 << 10)),
    error_on_excess_data: true,
    use_indices: false,
};

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("An error occurred on the underlying stream: {0}")]
    IO(#[source] io::Error),
    #[error("An error occurred while encoding/decoding a message: {0}")]
    Codec(#[source] serde_brief::Error),
}

impl From<io::Error> for MessageError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}
impl From<serde_brief::Error> for MessageError {
    fn from(value: serde_brief::Error) -> Self {
        Self::Codec(value)
    }
}

pub struct MessageCodec<T: Serialize + DeserializeOwned>(PhantomData<T>);

impl<T: Serialize + DeserializeOwned> Default for MessageCodec<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Serialize + DeserializeOwned> MessageCodec<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Serialize + DeserializeOwned> Encoder<T> for MessageCodec<T> {
    type Error = MessageError;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = serde_brief::to_vec_with_config(&item, SERDE_BRIEF_CFG.clone())?;
        let len = (data.len() as u32).to_le_bytes();

        dst.put(&len[..]);
        dst.put(&data[..]);
        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned> Decoder for MessageCodec<T> {
    type Item = T;
    type Error = MessageError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.remaining() < 4 {
            return Ok(None);
        }
        let len = src.copy_to_bytes(4).get_u32_le() as usize;
        let rem = src.remaining();
        if rem < 4 + len {
            return Ok(None);
        }
        src.advance(4);

        let data = {
            let buffer = src.split_to(len);
            serde_brief::from_reader_with_config::<_, T>(buffer.reader(), SERDE_BRIEF_CFG.clone())?
        };

        Ok(Some(data))
    }
}
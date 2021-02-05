use std::io;

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use super::message;

/// Wrapper type that covers all possible protobuf encoded Hyperion messages
#[derive(Debug)]
pub enum HyperionRequest {
    /// Solid color request
    ColorRequest(message::ColorRequest),
    /// Incoming image request
    ImageRequest(message::ImageRequest),
    /// Clear colors request
    ClearRequest(message::ClearRequest),
    /// Clear all colors request
    ClearAllRequest(message::HyperionRequest),
}

/// Error raised when parsing a protobuf encoded message fails
#[derive(Debug, Error)]
pub enum ProtoCodecError {
    /// I/O error
    #[error("I/O error: {}", 0)]
    IoError(#[from] io::Error),
    /// Protobuf decoding error
    #[error("decode error: {}", 0)]
    DecodeError(#[from] prost::DecodeError),
    /// Invalid incoming message
    #[error("invalid message")]
    InvalidMessageError,
    /// Protobuf encoding error
    #[error("encode error: {}", 0)]
    EncodeError(#[from] prost::EncodeError),
}

/// tokio Codec to decode incoming Hyperion protobuf messages
pub struct ProtoCodec {}

impl ProtoCodec {
    /// Create a new ProtoCodec
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for ProtoCodec {
    type Item = HyperionRequest;
    type Error = ProtoCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check that there is a size to be read
        if src.len() < 4 {
            return Ok(None);
        }

        let size = BigEndian::read_u32(&src[0..4]) as usize;

        // Check that we have the full message before decoding
        if src.len() - 4 < size {
            return Ok(None);
        }

        trace!("{} bytes message", size);

        // Attempt to parse using protobuf
        let range = &src[4..(4 + size)];
        let parsed = message::HyperionRequest::decode(range);

        // Process parse result
        let result = match parsed {
            Ok(message) => {
                // We parsed an HyperionMessage, which gives us the actual type
                // of the payload
                match message.command() {
                    message::hyperion_request::Command::Color => {
                        message::ColorRequest::decode(range).map(HyperionRequest::ColorRequest)
                    }
                    message::hyperion_request::Command::Image => {
                        message::ImageRequest::decode(range).map(HyperionRequest::ImageRequest)
                    }
                    message::hyperion_request::Command::Clear => {
                        message::ClearRequest::decode(range).map(HyperionRequest::ClearRequest)
                    }
                    message::hyperion_request::Command::Clearall => {
                        Ok(message).map(HyperionRequest::ClearAllRequest)
                    }
                }
                .map_err(|e| e.into())
            }
            Err(parse_error) => Err(ProtoCodecError::DecodeError(parse_error)),
        };

        // Consume the message from the buffer: since it's complete, the parsing
        // success does not depend on more data arriving
        src.advance(4 + size);

        result.map(Option::Some)
    }
}

impl Encoder<message::HyperionReply> for ProtoCodec {
    type Error = ProtoCodecError;

    fn encode(
        &mut self,
        item: message::HyperionReply,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        // Get the size of the message
        let message_size = item.encoded_len();

        // Reserve space in the dst buffer
        dst.reserve(4 + message_size as usize);

        // Write message size
        dst.put_u32(message_size as u32);

        // Write message contents
        item.encode(dst).map_err(ProtoCodecError::EncodeError)?;

        Ok(())
    }
}

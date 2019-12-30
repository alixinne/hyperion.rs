use error_chain::error_chain;

use tokio_util::codec::{Decoder, Encoder};

use super::message;

use bytes::{Buf, BufMut, BytesMut};

use prost::Message;

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

error_chain! {
    types {
        HyperionMessageError, HyperionMessageErrorKind, ResultExt;
    }

    foreign_links {
        Io(::std::io::Error);
        Decode(::prost::DecodeError);
        Encode(::prost::EncodeError);
    }

    errors {
        InvalidMessage {
            description("invalid message")
        }
    }
}

/// tokio Codec to decode incoming Hyperion protobuf messages
#[derive(Default)]
pub struct ProtoCodec {
    buf: Vec<u8>,
}

impl Decoder for ProtoCodec {
    type Item = HyperionRequest;
    type Error = HyperionMessageError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check that there is a size to be read
        if src.remaining() < 4 {
            return Ok(None);
        }

        // Peek at the size to see if we have enough
        let size = (&src[0..4]).get_u32() as usize;

        // Check that we have the full message before decoding
        if src.remaining() < size {
            return Ok(None);
        }

        // Consume size
        assert!(src.get_u32() as usize == size);
        trace!("{} bytes message", size);

        // Extend working buffer
        if size > self.buf.len() {
            self.buf.resize(size, 0);
        }

        // Copy message into buffer
        src.copy_to_slice(&mut self.buf[..]);

        // Attempt to parse using protobuf
        let parsed = message::HyperionRequest::decode(&self.buf[..]);

        // Process parse result
        let result = match parsed {
            Ok(message) => {
                // We parsed an HyperionMessage, which gives us the actual type
                // of the payload
                match message.command() {
                    message::hyperion_request::Command::Color => {
                        message.color_request.map(HyperionRequest::ColorRequest)
                    }
                    message::hyperion_request::Command::Image => {
                        message.image_request.map(HyperionRequest::ImageRequest)
                    }
                    message::hyperion_request::Command::Clear => {
                        message.clear_request.map(HyperionRequest::ClearRequest)
                    }
                    message::hyperion_request::Command::Clearall => {
                        Some(message).map(HyperionRequest::ClearAllRequest)
                    }
                }
                .ok_or_else(|| HyperionMessageErrorKind::InvalidMessage.into())
            }
            Err(parse_error) => Err(parse_error.into()),
        };

        result.map(Option::Some)
    }
}

impl Encoder for ProtoCodec {
    type Item = message::HyperionReply;
    type Error = HyperionMessageError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Get the size of the message
        let message_size = item.encoded_len();

        // Reserve space in the dst buffer
        dst.reserve(4 + message_size as usize);

        // Write message size
        dst.put_u32(message_size as u32);

        // Write message contents
        item.encode(dst)?;

        Ok(())
    }
}

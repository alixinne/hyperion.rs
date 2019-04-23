use bytes::BytesMut;
use tokio::codec::{Decoder, Encoder, LinesCodec};

use super::message;

use std::io;

/// Parse an incoming request as JSON into the corresponding message type
///
/// # Parameters
///
/// * `line`: input request line to parse as a message
///
/// # Errors
///
/// When the line cannot be parsed as JSON, the underlying error is returned from serde_json.
fn parse_request(line: &str) -> serde_json::Result<message::HyperionMessage> {
    serde_json::from_str(line)
}

/// Encode an outgoing reply as JSON
///
/// # Parameters
///
/// * `reply`: reply to encode as JSON
///
/// # Errors
///
/// When the reply cannot be encoded as JSON, the underlying error is returned from serde_json.
fn encode_reply(reply: &message::HyperionResponse) -> serde_json::Result<String> {
    serde_json::to_string(reply)
}

pub struct JsonCodec {
    lines: LinesCodec,
}

impl JsonCodec {
    pub fn new() -> Self {
        Self {
            lines: LinesCodec::new(),
        }
    }
}

impl Decoder for JsonCodec {
    type Item = message::HyperionMessage;
    type Error = failure::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.lines.decode(src) {
            Ok(lines_result) => Ok(match lines_result {
                Some(ref line) => Some(parse_request(line)?),
                None => None,
            }),
            Err(error) => Err(error.into()),
        }
    }
}

impl Encoder for JsonCodec {
    type Item = message::HyperionResponse;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match encode_reply(&item) {
            Ok(encoded) => self.lines.encode(encoded, dst),
            Err(encode_error) => Err(encode_error.into()),
        }
    }
}

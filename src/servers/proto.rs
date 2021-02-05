//! protobuf protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::prelude::*;
use prost::Message;
use thiserror::Error;
use tokio::net::TcpStream;

use crate::{
    global::{Global, InputMessage, InputMessageData, InputSourceHandle},
    image::{RawImage, RawImageError},
    models::Color,
};

/// Schema definitions as Serde serializable structures and enums
mod message;

use super::util::*;

#[derive(Debug, Error)]
pub enum ProtoServerError {
    #[error("i/o error: {0}")]
    Io(#[from] futures_io::Error),
    #[error("error decoding image: {0}")]
    ImageError(#[from] ImageError),
    #[error("error decoding image: {0}")]
    RawImageError(#[from] RawImageError),
    #[error("error broadcasting update: {0}")]
    Broadcast(#[from] tokio::sync::broadcast::error::SendError<InputMessage>),
    #[error("decode error: {}", 0)]
    DecodeError(#[from] prost::DecodeError),
}

fn encode_response(buf: &mut bytes::BytesMut, msg: impl prost::Message) -> bytes::Bytes {
    // Clear the buffer to start fresh
    buf.clear();

    // Reserve enough space for the response
    let len = msg.encoded_len();
    if buf.capacity() < len {
        buf.reserve(len * 2);
    }

    // Encode the message
    msg.encode(buf).unwrap();
    buf.split().freeze()
}

fn success_response(peer_addr: SocketAddr, buf: &mut bytes::BytesMut) -> bytes::Bytes {
    let mut reply = message::HyperionReply::default();
    reply.r#type = message::hyperion_reply::Type::Reply.into();
    reply.success = Some(true);

    trace!("({}) sending success: {:?}", peer_addr, reply);
    encode_response(buf, reply)
}

fn error_response(
    peer_addr: SocketAddr,
    buf: &mut bytes::BytesMut,
    error: impl std::fmt::Display,
) -> bytes::Bytes {
    let mut reply = message::HyperionReply::default();
    reply.r#type = message::hyperion_reply::Type::Reply.into();
    reply.success = Some(false);
    reply.error = Some(error.to_string());

    trace!("({}) sending error: {:?}", peer_addr, reply);
    encode_response(buf, reply)
}

fn handle_request(
    peer_addr: SocketAddr,
    request_bytes: bytes::BytesMut,
    source: &InputSourceHandle<InputMessage>,
) -> Result<(), ProtoServerError> {
    let request_bytes = request_bytes.freeze();
    let request = message::HyperionRequest::decode(request_bytes.clone())?;

    trace!("({}) got request: {:?}", peer_addr, request);

    match request.command() {
        message::hyperion_request::Command::Clearall => {
            // Update state
            source.send(InputMessageData::ClearAll)?;
        }

        message::hyperion_request::Command::Clear => {
            let clear_request = message::ClearRequest::decode(request_bytes)?;

            // Update state
            source.send(InputMessageData::Clear {
                priority: clear_request.priority,
            })?;
        }

        message::hyperion_request::Command::Color => {
            let color_request = message::ColorRequest::decode(request_bytes)?;

            let color = color_request.rgb_color;
            let color = (
                (color & 0x000_000FF) as u8,
                ((color & 0x0000_FF00) >> 8) as u8,
                ((color & 0x00FF_0000) >> 16) as u8,
            );

            // Update state
            source.send(InputMessageData::SolidColor {
                priority: color_request.priority,
                duration: i32_to_duration(color_request.duration),
                color: Color::from_components(color),
            })?;
        }

        message::hyperion_request::Command::Image => {
            let image_request = message::ImageRequest::decode(request_bytes)?;

            let width =
                u32::try_from(image_request.imagewidth).map_err(|_| ImageError::InvalidWidth)?;
            let height =
                u32::try_from(image_request.imageheight).map_err(|_| ImageError::InvalidHeight)?;
            let raw_image = RawImage::try_from((image_request.imagedata.to_vec(), width, height))?;

            // Update state
            source.send(InputMessageData::Image {
                priority: image_request.priority,
                duration: i32_to_duration(image_request.duration),
                image: Arc::new(raw_image),
            })?;
        }
    }

    Ok(())
}

pub async fn handle_client(
    (socket, peer_addr): (TcpStream, SocketAddr),
    global: Global,
) -> Result<(), ProtoServerError> {
    debug!("accepted new connection from {}", peer_addr);

    let framed = tokio_util::codec::LengthDelimitedCodec::builder()
        .length_field_length(4)
        .new_framed(socket);
    let (mut writer, mut reader) = framed.split();

    // unwrap: cannot fail because the priority is None
    let source = global
        .register_input_source(format!("Protobuf({})", peer_addr), None)
        .await
        .unwrap();

    // buffer for building responses
    let mut reply_buf = bytes::BytesMut::with_capacity(128);

    while let Some(request_bytes) = reader.next().await {
        let request_bytes = match request_bytes {
            Ok(rb) => rb,
            Err(error) => {
                error!("({}) error reading frame: {}", peer_addr, error);
                continue;
            }
        };

        let reply = match handle_request(peer_addr, request_bytes, &source) {
            Ok(()) => success_response(peer_addr, &mut reply_buf),
            Err(error) => {
                error!("({}) error processing request: {}", peer_addr, error);

                error_response(peer_addr, &mut reply_buf, error)
            }
        };

        writer.send(reply).await?;
    }

    Ok(())
}

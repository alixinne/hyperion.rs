//! protobuf protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::prelude::*;
use prost::Message;
use thiserror::Error;
use tokio::net::TcpStream;

use crate::{
    global::{Global, InputMessage},
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

/// Create a success response
///
/// # Parameters
///
/// `success`: true for a success, false for an error
fn success_response() -> bytes::Bytes {
    let mut reply = message::HyperionReply::default();
    reply.r#type = message::hyperion_reply::Type::Reply.into();
    reply.success = Some(true);

    let mut b = Vec::new();
    reply.encode(&mut b).unwrap();
    b.into()
}

fn error_response(error: impl std::fmt::Display) -> bytes::Bytes {
    let mut reply = message::HyperionReply::default();
    reply.r#type = message::hyperion_reply::Type::Reply.into();
    reply.success = Some(false);
    reply.error = Some(error.to_string());

    let mut b = Vec::new();
    reply.encode(&mut b).unwrap();
    b.into()
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
        .register_source(format!("Protobuf({})", peer_addr), None)
        .await
        .unwrap();

    while let Some(request_bytes) = reader.next().await {
        let request_bytes = match request_bytes {
            Ok(rb) => rb,
            Err(error) => {
                error!("({}) error reading frame: {}", peer_addr, error);
                continue;
            }
        };

        let request = match message::HyperionRequest::decode(request_bytes.clone()) {
            Ok(rq) => rq,
            Err(error) => {
                error!("({}) error decoding frame: {}", peer_addr, error);
                writer.send(error_response(error)).await?;
                continue;
            }
        };

        trace!("got request: {:?}", request);

        let reply = (|| -> Result<_, _> {
            match request.command() {
                message::hyperion_request::Command::Clearall => {
                    // Update state
                    source.send(InputMessage::ClearAll)?;

                    Ok(success_response())
                }

                message::hyperion_request::Command::Clear => {
                    let clear_request = message::ClearRequest::decode(request_bytes)?;

                    // Update state
                    source.send(InputMessage::Clear {
                        priority: clear_request.priority,
                    })?;

                    Ok(success_response())
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
                    source.send(InputMessage::SolidColor {
                        priority: color_request.priority,
                        duration: i32_to_duration(color_request.duration),
                        color: Color::from_components(color),
                    })?;

                    Ok(success_response())
                }

                message::hyperion_request::Command::Image => {
                    let image_request = message::ImageRequest::decode(request_bytes)?;

                    let data = image_request.imagedata;
                    let width = image_request.imagewidth;
                    let height = image_request.imageheight;
                    let priority = image_request.priority;
                    let duration = image_request.duration;

                    let width = u32::try_from(width).map_err(|_| ImageError::InvalidWidth)?;
                    let height = u32::try_from(height).map_err(|_| ImageError::InvalidHeight)?;
                    let raw_image = RawImage::try_from((data.to_vec(), width, height))?;

                    // Update state
                    source.send(InputMessage::Image {
                        priority,
                        duration: i32_to_duration(duration),
                        image: Arc::new(raw_image),
                    })?;

                    Ok(success_response())
                }
            }
        })();

        let reply = match reply {
            Ok(res) => res,
            Err(ProtoServerError::Broadcast(b)) => {
                return Err(ProtoServerError::Broadcast(b));
            }
            Err(error) => error_response(error),
        };

        trace!("sending response: {:?}", reply);
        writer.send(reply).await?;
    }

    Ok(())
}

//! protobuf protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::prelude::*;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    global::{Global, InputMessage},
    image::RawImage,
    models::Color,
};

/// Schema definitions as Serde serializable structures and enums
mod message;

/// Protobuf protocol codec definition
mod codec;
use codec::*;

#[derive(Debug, Error)]
pub enum ProtoServerError {
    #[error("i/o error: {0}")]
    Io(#[from] futures_io::Error),
    #[error("codec error: {0}")]
    Codec(#[from] ProtoCodecError),
    #[error("error broadcasting update: {0}")]
    Broadcast(#[from] tokio::sync::broadcast::error::SendError<InputMessage>),
}

/// Create a success response
///
/// # Parameters
///
/// `success`: true for a success, false for an error
fn success_response() -> message::HyperionReply {
    let mut reply = message::HyperionReply::default();
    reply.r#type = message::hyperion_reply::Type::Reply.into();
    reply.success = Some(true);

    reply
}

fn error_response(error: impl std::fmt::Display) -> message::HyperionReply {
    let mut reply = message::HyperionReply::default();
    reply.r#type = message::hyperion_reply::Type::Reply.into();
    reply.success = Some(false);
    reply.error = Some(error.to_string());

    reply
}

fn i32_to_duration(d: Option<i32>) -> Option<chrono::Duration> {
    if let Some(d) = d {
        if d <= 0 {
            None
        } else {
            Some(chrono::Duration::milliseconds(d as _))
        }
    } else {
        None
    }
}

pub async fn handle_client(
    (socket, peer_addr): (TcpStream, SocketAddr),
    global: Global,
) -> Result<(), ProtoServerError> {
    debug!("accepted new connection from {}", peer_addr);

    let sender = global.read().await.input_tx.clone();

    let framed = Framed::new(socket, ProtoCodec::new());
    let (mut writer, mut reader) = framed.split();

    while let Some(request) = reader.next().await {
        trace!("got request: {:?}", request);

        let reply = match request {
            Ok(HyperionRequest::ClearAllRequest(_)) => {
                // Update state
                sender.send(InputMessage::ClearAll)?;

                success_response()
            }

            Ok(HyperionRequest::ClearRequest(clear_request)) => {
                // Update state
                sender.send(InputMessage::Clear {
                    priority: clear_request.priority,
                })?;

                success_response()
            }

            Ok(HyperionRequest::ColorRequest(color_request)) => {
                let color = color_request.rgb_color;
                let color = (
                    (color & 0x000_000FF) as u8,
                    ((color & 0x0000_FF00) >> 8) as u8,
                    ((color & 0x00FF_0000) >> 16) as u8,
                );

                // Update state
                sender.send(InputMessage::SolidColor {
                    priority: color_request.priority,
                    duration: i32_to_duration(color_request.duration),
                    color: Color::from_components(color),
                })?;

                success_response()
            }

            Ok(HyperionRequest::ImageRequest(image_request)) => {
                let data = image_request.imagedata;
                let width = image_request.imagewidth;
                let height = image_request.imageheight;
                let priority = image_request.priority;
                let duration = image_request.duration;

                #[derive(Debug, Error)]
                enum ImageError {
                    #[error("invalid width")]
                    InvalidWidth,
                    #[error("invalid height")]
                    InvalidHeight,
                    #[error("image error: {0}")]
                    Image(#[from] crate::image::RawImageError),
                    #[error("error broadcasting update: {0}")]
                    Broadcast(#[from] tokio::sync::broadcast::error::SendError<InputMessage>),
                }

                match (|| -> Result<_, ImageError> {
                    let width = u32::try_from(width).map_err(|_| ImageError::InvalidWidth)?;
                    let height = u32::try_from(height).map_err(|_| ImageError::InvalidHeight)?;
                    let raw_image = RawImage::try_from((data.to_vec(), width, height))?;

                    // Update state
                    sender.send(InputMessage::Image {
                        priority,
                        duration: i32_to_duration(duration),
                        image: Arc::new(raw_image),
                    })?;

                    Ok(())
                })() {
                    Ok(_) => success_response(),
                    Err(ImageError::Broadcast(b)) => return Err(ProtoServerError::Broadcast(b)),
                    Err(error) => error_response(error),
                }
            }

            Err(error) => {
                error!("{}", error);
                error_response(error)
            }
        };

        trace!("sending response: {:?}", reply);
        writer.send(reply).await?;
    }

    Ok(())
}

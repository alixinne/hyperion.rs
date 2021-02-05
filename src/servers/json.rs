//! JSON protocol server implementation

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
use message::{HyperionMessage, HyperionResponse};

/// JSON protocol codec definition
mod codec;
use codec::*;

#[derive(Debug, Error)]
pub enum JsonServerError {
    #[error("i/o error: {0}")]
    Io(#[from] futures_io::Error),
    #[error("codec error: {0}")]
    Codec(#[from] JsonCodecError),
    #[error("error broadcasting update: {0}")]
    Broadcast(#[from] tokio::sync::broadcast::error::SendError<InputMessage>),
}

pub async fn handle_client(
    (socket, peer_addr): (TcpStream, SocketAddr),
    global: Global,
) -> Result<(), JsonServerError> {
    let sender = global.read().await.input_tx.clone();

    debug!("accepted new connection from {}", peer_addr,);

    let framed = Framed::new(socket, JsonCodec::new());
    let (mut writer, mut reader) = framed.split();

    while let Some(request) = reader.next().await {
        trace!("processing request: {:?}", request);

        let reply = match request {
            Ok(HyperionMessage::ClearAll) => {
                // Update state
                sender.send(InputMessage::ClearAll)?;

                HyperionResponse::SuccessResponse { success: true }
            }

            Ok(HyperionMessage::Clear { priority }) => {
                // Update state
                sender.send(InputMessage::Clear { priority })?;

                HyperionResponse::SuccessResponse { success: true }
            }

            Ok(HyperionMessage::Color {
                priority,
                duration,
                color,
            }) => {
                // Update state
                sender.send(InputMessage::SolidColor {
                    priority,
                    duration: duration.map(|ms| chrono::Duration::milliseconds(ms as _)),
                    color: Color::from_components((color[0], color[1], color[2])),
                })?;

                HyperionResponse::SuccessResponse { success: true }
            }

            Ok(HyperionMessage::Image {
                priority,
                duration,
                imagewidth,
                imageheight,
                imagedata,
            }) => match RawImage::try_from((imagedata, imagewidth, imageheight)) {
                Ok(raw_image) => {
                    sender.send(InputMessage::Image {
                        priority,
                        duration: duration.map(|ms| chrono::Duration::milliseconds(ms as _)),
                        image: Arc::new(raw_image),
                    })?;

                    HyperionResponse::SuccessResponse { success: true }
                }
                Err(error) => HyperionResponse::ErrorResponse {
                    success: false,
                    error: error.to_string(),
                },
            },

            Err(error) => HyperionResponse::ErrorResponse {
                success: false,
                error: error.to_string(),
            },

            _ => HyperionResponse::ErrorResponse {
                success: false,
                error: "not implemented".into(),
            },
        };

        trace!("sending response: {:?}", reply);

        writer.send(reply).await?;
    }

    Ok(())
}

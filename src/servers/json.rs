//! JSON protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::prelude::*;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    global::{Global, InputMessage, InputMessageData, InputSourceHandle},
    image::{RawImage, RawImageError},
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
    #[error("request not implemented")]
    NotImplemented,
    #[error("error decoding image")]
    Image(#[from] RawImageError),
}

fn handle_request(
    request: Result<HyperionMessage, JsonCodecError>,
    source: &InputSourceHandle<InputMessage>,
) -> Result<Option<HyperionResponse>, JsonServerError> {
    match request? {
        HyperionMessage::ClearAll => {
            // Update state
            source.send(InputMessageData::ClearAll)?;
        }

        HyperionMessage::Clear { priority } => {
            // Update state
            source.send(InputMessageData::Clear { priority })?;
        }

        HyperionMessage::Color {
            priority,
            duration,
            color,
        } => {
            // Update state
            source.send(InputMessageData::SolidColor {
                priority,
                duration: duration.map(|ms| chrono::Duration::milliseconds(ms as _)),
                color: Color::from_components((color[0], color[1], color[2])),
            })?;
        }

        HyperionMessage::Image {
            priority,
            duration,
            imagewidth,
            imageheight,
            imagedata,
        } => {
            let raw_image = RawImage::try_from((imagedata, imagewidth, imageheight))?;

            source.send(InputMessageData::Image {
                priority,
                duration: duration.map(|ms| chrono::Duration::milliseconds(ms as _)),
                image: Arc::new(raw_image),
            })?;
        }

        HyperionMessage::ServerInfo => {
            // Just answer the serverinfo request, no need to update state
            return Ok(Some(HyperionResponse::server_info(vec![])));
        }

        _ => return Err(JsonServerError::NotImplemented),
    };

    Ok(None)
}

pub async fn handle_client(
    (socket, peer_addr): (TcpStream, SocketAddr),
    global: Global,
) -> Result<(), JsonServerError> {
    debug!("accepted new connection from {}", peer_addr,);

    let framed = Framed::new(socket, JsonCodec::new());
    let (mut writer, mut reader) = framed.split();

    // unwrap: cannot fail because the priority is None
    let source = global
        .register_input_source(format!("JSON({})", peer_addr), None)
        .await
        .unwrap();

    while let Some(request) = reader.next().await {
        trace!("({}) processing request: {:?}", peer_addr, request);

        let reply = match handle_request(request, &source) {
            Ok(None) => HyperionResponse::success(),
            Ok(Some(response)) => response,
            Err(error) => {
                error!("({}) error processing request: {}", peer_addr, error);

                HyperionResponse::error(&error)
            }
        };

        trace!("({}) sending response: {:?}", peer_addr, reply);

        writer.send(reply).await?;
    }

    Ok(())
}

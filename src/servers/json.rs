//! JSON protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::prelude::*;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use validator::Validate;

use crate::{
    global::{Global, InputMessage, InputMessageData, InputSourceHandle},
    image::{RawImage, RawImageError},
};

/// Schema definitions as Serde serializable structures and enums
mod message;
use message::{HyperionCommand, HyperionMessage, HyperionResponse};

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
    #[error("error validating request: {0}")]
    Validation(#[from] validator::ValidationErrors),
}

fn handle_request(
    request: HyperionMessage,
    source: &InputSourceHandle<InputMessage>,
) -> Result<Option<HyperionResponse>, JsonServerError> {
    request.validate()?;

    match request.command {
        HyperionCommand::ClearAll => {
            // Update state
            source.send(InputMessageData::ClearAll)?;
        }

        HyperionCommand::Clear(message::Clear { priority }) => {
            // Update state
            source.send(InputMessageData::Clear { priority })?;
        }

        HyperionCommand::Color(message::Color {
            priority,
            duration,
            color,
            origin: _,
        }) => {
            // TODO: Handle origin field

            // Update state
            source.send(InputMessageData::SolidColor {
                priority,
                duration: duration.map(|ms| chrono::Duration::milliseconds(ms as _)),
                color,
            })?;
        }

        HyperionCommand::Image(message::Image {
            priority,
            duration,
            imagewidth,
            imageheight,
            imagedata,
            origin: _,
            format: _,
            scale: _,
            name: _,
        }) => {
            // TODO: Handle origin, format, scale, name fields

            let raw_image = RawImage::try_from((imagedata, imagewidth, imageheight))?;

            source.send(InputMessageData::Image {
                priority,
                duration: duration.map(|ms| chrono::Duration::milliseconds(ms as _)),
                image: Arc::new(raw_image),
            })?;
        }

        HyperionCommand::ServerInfo(message::ServerInfoRequest { subscribe: _ }) => {
            // TODO: Handle subscribe field

            // Just answer the serverinfo request, no need to update state
            return Ok(Some(HyperionResponse::server_info(request.tan, vec![])));
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

        let mut tan = None;
        let reply = match request.map_err(JsonServerError::from).and_then(|request| {
            tan = request.tan;
            handle_request(request, &source)
        }) {
            Ok(None) => HyperionResponse::success(tan),
            Ok(Some(response)) => response,
            Err(error) => {
                error!("({}) error processing request: {}", peer_addr, error);

                HyperionResponse::error(tan, &error)
            }
        };

        trace!("({}) sending response: {:?}", peer_addr, reply);

        writer.send(reply).await?;
    }

    Ok(())
}

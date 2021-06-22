//! flatbuffers flatcol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::prelude::*;
use thiserror::Error;
use tokio::net::TcpStream;

use crate::{
    global::{Global, InputMessage, InputMessageData, InputSourceHandle},
    image::{RawImage, RawImageError},
    models::Color,
};

use super::util::*;

/// Schema definitions as Serde serializable structures and enums
mod message;

#[derive(Debug, Error)]
pub enum FlatServerError {
    #[error("i/o error: {0}")]
    Io(#[from] futures_io::Error),
    #[error("error broadcasting update: {0}")]
    Broadcast(#[from] tokio::sync::broadcast::error::SendError<InputMessage>),
    #[error("source not registered")]
    Unregistered,
    #[error("invalid priority for registration, should be in [100, 200), got {0}")]
    InvalidPriority(i32),
    #[error("source already registered")]
    AlreadyRegistered,
    #[error("unknown command")]
    UnknownCommand,
    #[error("invalid raw image data")]
    RawImageError(#[from] RawImageError),
    #[error("error decoding image data")]
    Image(#[from] ImageError),
    #[error("error decoding frame")]
    FlatBuffer(#[from] flatbuffers::InvalidFlatbuffer),
}

fn register_response(builder: &mut flatbuffers::FlatBufferBuilder, priority: i32) -> bytes::Bytes {
    let mut reply = message::ReplyBuilder::new(builder);
    reply.add_registered(priority);

    let reply = reply.finish();

    builder.finish(reply, None);
    bytes::Bytes::copy_from_slice(builder.finished_data())
}

fn error_response(
    builder: &mut flatbuffers::FlatBufferBuilder,
    error: impl std::fmt::Display,
) -> bytes::Bytes {
    let error = builder.create_string(error.to_string().as_str());

    let mut reply = message::ReplyBuilder::new(builder);
    reply.add_error(error);

    let reply = reply.finish();

    builder.finish(reply, None);
    bytes::Bytes::copy_from_slice(builder.finished_data())
}

async fn handle_request(
    peer_addr: SocketAddr,
    request_bytes: bytes::BytesMut,
    source: &mut Option<InputSourceHandle<InputMessage>>,
    global: &Global,
) -> Result<(), FlatServerError> {
    let request = message::root_as_request(request_bytes.as_ref())?;

    trace!("({}) got request: {:?}", peer_addr, request.command_type());

    if let Some(source) = source.as_ref() {
        // unwrap: we set a priority when we got the register call
        let priority = source.priority().unwrap();

        if let Some(clear) = request.command_as_clear() {
            // Update state
            if clear.priority() < 0 {
                source.send(InputMessageData::ClearAll)?;
            } else {
                source.send(InputMessageData::Clear {
                    priority: clear.priority(),
                })?;
            }
        } else if let Some(color) = request.command_as_color() {
            let rgb = color.data();
            let rgb = (
                (rgb & 0x000_000FF) as u8,
                ((rgb & 0x0000_FF00) >> 8) as u8,
                ((rgb & 0x00FF_0000) >> 16) as u8,
            );

            // Update state
            source.send(InputMessageData::SolidColor {
                // TODO
                priority: 0,
                duration: i32_to_duration(Some(color.duration())),
                color: Color::from_components(rgb),
            })?;
        } else if let Some(image) = request.command_as_image() {
            // Get raw image
            let data = image
                .data_as_raw_image()
                .ok_or_else(|| ImageError::RawImageMissing)?;

            // Extract fields
            let duration = image.duration();
            let width = data.width();
            let height = data.height();
            let data = data.data().ok_or_else(|| ImageError::RawImageMissing)?;

            // Parse message
            let width = u32::try_from(width).map_err(|_| ImageError::InvalidWidth)?;
            let height = u32::try_from(height).map_err(|_| ImageError::InvalidHeight)?;
            let raw_image = RawImage::try_from((data.to_vec(), width, height))?;

            // Update state
            source.send(InputMessageData::Image {
                priority,
                duration: i32_to_duration(Some(duration)),
                image: Arc::new(raw_image),
            })?;
        } else if let Some(_) = request.command_as_register() {
            return Err(FlatServerError::AlreadyRegistered);
        } else {
            return Err(FlatServerError::UnknownCommand);
        }
    } else {
        if let Some(register) = request.command_as_register() {
            let priority = register.priority();

            if priority < 100 || priority >= 200 {
                return Err(FlatServerError::InvalidPriority(priority));
            } else {
                // unwrap: we checked the priority value before
                *source = Some(
                    global
                        .register_input_source(
                            format!("FlatBuffers({}): {}", peer_addr, register.origin()),
                            Some(priority),
                        )
                        .await
                        .unwrap(),
                );
            }
        } else {
            return Err(FlatServerError::Unregistered);
        }
    };

    Ok(())
}

pub async fn handle_client(
    (socket, peer_addr): (TcpStream, SocketAddr),
    global: Global,
) -> Result<(), FlatServerError> {
    debug!("accepted new connection from {}", peer_addr);

    let framed = tokio_util::codec::LengthDelimitedCodec::builder()
        .length_field_length(4)
        .new_framed(socket);

    let (mut writer, mut reader) = framed.split();

    let mut source = None;
    let mut builder = flatbuffers::FlatBufferBuilder::new();

    while let Some(request_bytes) = reader.next().await {
        let request_bytes = match request_bytes {
            Ok(rb) => rb,
            Err(error) => {
                error!("({}) error reading frame: {}", peer_addr, error);
                continue;
            }
        };

        builder.reset();

        let reply = match handle_request(peer_addr, request_bytes, &mut source, &global).await {
            Ok(()) => {
                if let Some(source) = source.as_ref() {
                    register_response(&mut builder, source.priority().unwrap())
                } else {
                    error_response(&mut builder, "unregistered source")
                }
            }
            Err(error) => {
                error!("({}) error processing request: {}", peer_addr, error);

                error_response(&mut builder, error)
            }
        };

        trace!("sending response: {:?}", reply);
        writer.send(reply).await?;
    }

    Ok(())
}
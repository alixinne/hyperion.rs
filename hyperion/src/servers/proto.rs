//! protobuf protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

use crate::color;
use crate::hyperion::{Input, StateUpdate};
use crate::image::RawImage;

/// Schema definitions as Serde serializable structures and enums
mod message;

/// Protobuf protocol codec definition
mod codec;
use codec::*;

#[allow(missing_docs)]
mod errors {
    use error_chain::error_chain;

    error_chain! {
        types {
            ProtoServerError, ProtoServerErrorKind, ResultExt;
        }

        foreign_links {
            Io(::std::io::Error);
        }
    }
}

pub use errors::*;

/// Create a success response
fn success_response() -> message::HyperionReply {
    let mut reply = message::HyperionReply::default();
    reply.set_type(message::hyperion_reply::Type::Reply);
    reply.success = Some(true);

    reply
}

/// Create an error response
fn error_response(error: impl ToString) -> message::HyperionReply {
    let mut reply = message::HyperionReply::default();
    reply.set_type(message::hyperion_reply::Type::Reply);
    reply.success = Some(false);
    reply.error = Some(error.to_string());

    reply
}

async fn process(
    mut tx: mpsc::Sender<Input>,
    socket: TcpStream,
    peer_addr: SocketAddr,
) -> Result<(), mpsc::error::SendError<Input>> {
    let mut framed = Framed::new(socket, ProtoCodec::default());

    while let Some(request) = framed.next().await {
        trace!("processing request: {:?}", request);

        let reply = match request {
            Ok(HyperionRequest::ClearAllRequest(_)) => {
                // Update state
                tx.send(Input::user_input(StateUpdate::clear(), 0, None))
                    .await?;

                success_response()
            }
            Ok(HyperionRequest::ClearRequest(clear_request)) => {
                // Update state
                tx.send(Input::user_input(
                    StateUpdate::clear(),
                    clear_request.priority,
                    None,
                ))
                .await?;

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
                tx.send(Input::user_input(
                    StateUpdate::solid(color::ColorPoint::from((color.0, color.1, color.2))),
                    color_request.priority,
                    color_request.duration,
                ))
                .await?;

                success_response()
            }
            Ok(HyperionRequest::ImageRequest(image_request)) => {
                let data = image_request.imagedata;
                let width = image_request.imagewidth;
                let height = image_request.imageheight;
                let priority = image_request.priority;
                let duration = image_request.duration;

                // Try to convert sizes to unsigned fields
                if let Ok(imagewidth) = u32::try_from(width) {
                    if let Ok(imageheight) = u32::try_from(height) {
                        // Try to create image from raw data and given size
                        if let Ok(raw_image) = RawImage::try_from((data, imagewidth, imageheight)) {
                            // Update state
                            tx.send(Input::user_input(
                                StateUpdate::image(raw_image),
                                priority,
                                duration,
                            ))
                            .await?;

                            success_response()
                        } else {
                            error_response("invalid image data")
                        }
                    } else {
                        error_response("invalid image height")
                    }
                } else {
                    error_response("invalid image width")
                }
            }
            Err(error) => {
                if let HyperionMessageErrorKind::Io(io_error) = error.kind() {
                    if let std::io::ErrorKind::ConnectionReset = io_error.kind() {
                        // Client disconnect
                        info!("proto({}): client disconnected", peer_addr);
                        break;
                    }
                }

                warn!("proto({}): {:?}", peer_addr, error);
                error_response(error)
            }
        };

        trace!("sending response: {:?}", reply);

        if let Err(error) = framed.send(reply).await {
            warn!(
                "proto({}): disconnecting client because of error: {:?}",
                peer_addr, error
            );
            break;
        }
    }

    Ok(())
}

/// Binds the protobuf protocol server implementation to the given address, returning a future to
/// process incoming requests.
///
/// # Parameters
///
/// * `address`: address (and port) of the endpoint to bind the protobuf server to
///
/// # Errors
///
/// * When the server can't be bound to the given address
pub async fn bind(
    address: impl tokio::net::ToSocketAddrs,
) -> Result<mpsc::Receiver<Input>, ProtoServerError> {
    let mut listener = TcpListener::bind(address).await?;
    info!("server listening on {}", listener.local_addr().unwrap());

    // Capacity is 1 because all inputs have to be processed as soon as possible
    // which means buffering makes no sense
    let (tx, rx) = mpsc::channel(1);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    debug!("accepted new connection from {}", peer_addr);
                    tokio::spawn(process(tx.clone(), socket, peer_addr));
                }
                Err(error) => {
                    error!("accept error: {:?}", error);
                }
            }
        }
    });

    Ok(rx)
}

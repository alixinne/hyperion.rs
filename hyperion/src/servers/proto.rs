//! protobuf protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio::codec::Framed;

use crate::color;
use crate::hyperion::{Input, StateUpdate};
use crate::image::RawImage;
use crate::runtime::HostHandle;

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
///
/// # Parameters
///
/// `success`: true for a success, false for an error
fn success_response(success: bool) -> message::HyperionReply {
    let mut reply = message::HyperionReply::default();
    reply.set_type(message::hyperion_reply::Type::Reply);
    reply.success = Some(success);

    reply
}

/// Binds the protobuf protocol server implementation to the given address, returning a future to
/// process incoming requests.
///
/// # Parameters
///
/// * `address`: address (and port) of the endpoint to bind the protobuf server to
/// * `host`: component host
/// * `tripwire`: handle to the cancellation future
///
/// # Errors
///
/// * When the server can't be bound to the given address
pub fn bind(
    address: &SocketAddr,
    host: HostHandle,
    tripwire: stream_cancel::Tripwire,
) -> Result<Box<dyn Future<Item = (), Error = std::io::Error> + Send>, ProtoServerError> {
    let listener = TcpListener::bind(&address)?;

    let server = listener.incoming().for_each(move |socket| {
        debug!(
            "accepted new connection from {}",
            socket.peer_addr().unwrap()
        );

        let sender = host.get_service_input_sender();

        let framed = Framed::new(socket, ProtoCodec::new());
        let (writer, reader) = framed.split();

        let action = reader
            .map(move |request| {
                trace!("got request: {:?}", request);

                let reply = match request {
                    HyperionRequest::ClearAllRequest(_) => {
                        // Update state
                        sender
                            .unbounded_send(Input::user_input(StateUpdate::Clear, 0, None))
                            .unwrap();

                        success_response(true)
                    }
                    HyperionRequest::ClearRequest(clear_request) => {
                        // Update state
                        sender
                            .unbounded_send(Input::user_input(
                                StateUpdate::Clear,
                                clear_request.priority,
                                None,
                            ))
                            .unwrap();

                        success_response(true)
                    }
                    HyperionRequest::ColorRequest(color_request) => {
                        let color = color_request.rgb_color;
                        let color = (
                            (color & 0x000_000FF) as u8,
                            ((color & 0x0000_FF00) >> 8) as u8,
                            ((color & 0x00FF_0000) >> 16) as u8,
                        );

                        // Update state
                        sender
                            .unbounded_send(Input::user_input(
                                StateUpdate::SolidColor {
                                    color: color::ColorPoint::from((color.0, color.1, color.2)),
                                },
                                color_request.priority,
                                color_request.duration,
                            ))
                            .unwrap();

                        success_response(true)
                    }
                    HyperionRequest::ImageRequest(image_request) => {
                        let data = image_request.imagedata;
                        let width = image_request.imagewidth;
                        let height = image_request.imageheight;
                        let priority = image_request.priority;
                        let duration = image_request.duration;

                        // Try to convert sizes to unsigned fields
                        u32::try_from(width)
                            .and_then(|imagewidth| {
                                u32::try_from(height).map(|imageheight| (imagewidth, imageheight))
                            })
                            .map_err(|_| "invalid size".to_owned())
                            .and_then(|(imagewidth, imageheight)| {
                                // Try to create image from raw data and given size
                                RawImage::try_from((data, imagewidth, imageheight))
                                    .map(|raw_image| {
                                        // Update state
                                        sender
                                            .unbounded_send(Input::user_input(
                                                StateUpdate::Image(raw_image),
                                                priority,
                                                duration,
                                            ))
                                            .unwrap();

                                        success_response(true)
                                    })
                                    .map_err(|error| error.to_string())
                            })
                            .unwrap_or_else(|_error| success_response(false))
                    }
                };

                trace!("sending response: {:?}", reply);
                reply
            })
            .forward(writer)
            .map(|_| {})
            .map_err(|e| {
                warn!("error while processing request: {}", e);
            })
            .select(tripwire.clone())
            .map(|_| ())
            .map_err(|_| {
                error!("server tripwire error");
            });

        tokio::spawn(action);

        Ok(())
    });

    info!("server listening on {}", address);

    Ok(Box::new(server))
}

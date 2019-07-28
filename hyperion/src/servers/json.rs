//! JSON protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio::codec::Framed;

use crate::color;
use crate::hyperion::{Input, StateUpdate};
use crate::image::RawImage;

use futures::sync::mpsc;

/// Schema definitions as Serde serializable structures and enums
mod message;
use message::{HyperionMessage, HyperionResponse};

/// JSON protocol codec definition
mod codec;
use codec::*;

/// Binds the JSON protocol server implementation to the given address, returning a future to
/// process incoming requests.
///
/// # Parameters
///
/// * `address`: address (and port) of the endpoint to bind the JSON server to
/// * `sender`: channel endpoint to send state updates to
/// * `tripwire`: handle to the cancellation future
///
/// # Errors
///
/// * When the server can't be bound to the given address
pub fn bind(
    address: &SocketAddr,
    sender: mpsc::UnboundedSender<Input>,
    tripwire: stream_cancel::Tripwire,
) -> Result<Box<dyn Future<Item = (), Error = std::io::Error> + Send>, failure::Error> {
    let listener = TcpListener::bind(&address)?;

    let server = listener.incoming().for_each(move |socket| {
        debug!(
            "accepted new connection from {}",
            socket.peer_addr().unwrap()
        );

        let sender = sender.clone();

        let framed = Framed::new(socket, JsonCodec::new());
        let (writer, reader) = framed.split();

        let action = reader
            .and_then(move |request| {
                trace!("processing request: {:?}", request);

                let reply = match request {
                    HyperionMessage::ClearAll => {
                        // Update state
                        sender
                            .unbounded_send(Input::new(StateUpdate::Clear))
                            .unwrap();

                        HyperionResponse::SuccessResponse { success: true }
                    }
                    HyperionMessage::Clear { priority } => {
                        // Update state
                        sender
                            .unbounded_send(Input::from_priority(StateUpdate::Clear, priority))
                            .unwrap();

                        HyperionResponse::SuccessResponse { success: true }
                    }
                    HyperionMessage::Color {
                        priority,
                        duration,
                        color,
                    } => {
                        let update = StateUpdate::SolidColor {
                            color: color::ColorPoint::from_rgb((
                                f32::from(color[0]) / 255.0,
                                f32::from(color[1]) / 255.0,
                                f32::from(color[2]) / 255.0,
                            )),
                        };

                        // Update state
                        sender
                            .unbounded_send(Input::from_full(update, priority, duration))
                            .unwrap();

                        HyperionResponse::SuccessResponse { success: true }
                    }
                    HyperionMessage::Image {
                        priority,
                        duration,
                        imagewidth,
                        imageheight,
                        imagedata,
                    } => {
                        // Try to convert sizes to unsigned fields
                        u32::try_from(imagewidth)
                            .and_then(|imagewidth| {
                                u32::try_from(imageheight)
                                    .map(|imageheight| (imagewidth, imageheight))
                            })
                            .map_err(|_| "invalid size".to_owned())
                            .and_then(|(imagewidth, imageheight)| {
                                // Try to create image from raw data and given size
                                RawImage::try_from((imagedata, imagewidth, imageheight))
                                    .map(|raw_image| {
                                        // Update state
                                        sender
                                            .unbounded_send(Input::from_full(
                                                StateUpdate::Image(raw_image),
                                                priority,
                                                duration,
                                            ))
                                            .unwrap();

                                        HyperionResponse::SuccessResponse { success: true }
                                    })
                                    .map_err(|error| error.to_string())
                            })
                            .unwrap_or_else(|error| HyperionResponse::ErrorResponse {
                                success: false,
                                error,
                            })
                    }
                    _ => HyperionResponse::ErrorResponse {
                        success: false,
                        error: "not implemented".into(),
                    },
                };

                trace!("sending response: {:?}", reply);

                Ok(reply)
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

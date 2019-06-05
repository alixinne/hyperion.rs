//! JSON protocol server implementation

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio::codec::Framed;

use crate::color;
use crate::hyperion::StateUpdate;

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
    sender: mpsc::UnboundedSender<StateUpdate>,
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
                    HyperionMessage::ClearAll | HyperionMessage::Clear { .. } => {
                        // TODO: clear: handle priority

                        // Update state
                        sender.unbounded_send(StateUpdate::ClearAll).unwrap();

                        HyperionResponse::SuccessResponse { success: true }
                    }
                    HyperionMessage::Color { color, .. } => {
                        // TODO: color: handle priority and duration

                        // Update state
                        sender
                            .unbounded_send(StateUpdate::SolidColor {
                                color: color::ColorPoint::from_rgb((
                                    f32::from(color[0]) / 255.0,
                                    f32::from(color[1]) / 255.0,
                                    f32::from(color[2]) / 255.0,
                                )),
                            })
                            .unwrap();

                        HyperionResponse::SuccessResponse { success: true }
                    }
                    HyperionMessage::Image {
                        imagewidth,
                        imageheight,
                        imagedata,
                        ..
                    } => {
                        // TODO: image: handle priority and duration

                        // Update state
                        sender
                            .unbounded_send(StateUpdate::Image {
                                data: imagedata,
                                width: imagewidth as u32,
                                height: imageheight as u32,
                            })
                            .unwrap();

                        HyperionResponse::SuccessResponse { success: true }
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

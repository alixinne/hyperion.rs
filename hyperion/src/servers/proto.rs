//! protobuf protocol server implementation

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio_codec::Framed;

use crate::hyperion::StateUpdate;
use futures::sync::mpsc;

/// Schema definitions as Serde serializable structures and enums
mod message;

/// Protobuf protocol codec definition
mod codec;
use codec::*;

fn success_response(success: bool) -> message::HyperionReply {
    let mut reply = message::HyperionReply::new();
    reply.set_field_type(message::HyperionReply_Type::REPLY);
    reply.set_success(success);

    reply
}

/// Binds the protobuf protocol server implementation to the given address, returning a future to
/// process incoming requests.
///
/// # Parameters
///
/// * `address`: address (and port) of the endpoint to bind the protobuf server to
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

        let framed = Framed::new(socket, ProtoCodec::new());
        let (writer, reader) = framed.split();

        let action = reader
            .map(move |request| {
                debug!("got request: {:?}", request);

                let reply = match request {
                    HyperionRequest::ClearAllRequest(_) => {
                        // Update state
                        sender.unbounded_send(StateUpdate::ClearAll).unwrap();

                        success_response(true)
                    },
                    HyperionRequest::ColorRequest(color_request) => {
                        let color = color_request.get_RgbColor();
                        let color = (
                            color & 0x000_000FF,
                            (color & 0x0000_FF00) >> 8,
                            (color & 0x00FF_0000) >> 16
                        );

                        // Update state
                        sender.unbounded_send(StateUpdate::SolidColor {
                            color: palette::LinSrgb::from_components((
                                color.0 as f32 / 255.0,
                                color.1 as f32 / 255.0,
                                color.2 as f32 / 255.0,
                            )),
                        }).unwrap();

                        success_response(true)
                    },
                    _ => success_response(false),
                };

                debug!("sending response: {:?}", reply);
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

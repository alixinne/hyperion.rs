//! protobuf protocol server implementation

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio_codec::Framed;

/// Schema definitions as Serde serializable structures and enums
mod message;

/// Protobuf protocol codec definition
mod codec;
use codec::*;

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
pub fn bind(address: &SocketAddr) -> Result<Box<dyn Future<Item = (), Error = std::io::Error> + Send>, failure::Error> {
    let listener = TcpListener::bind(&address)?;

    let server = listener
        .incoming()
        .for_each(|socket| {
            debug!(
                "accepted new connection from {}",
                socket.peer_addr().unwrap()
            );

            let framed = Framed::new(socket, ProtoCodec::new());
            let (writer, reader) = framed.split();

            let action = reader
                .map(|request| {
                    debug!("got request: {:?}", request);

                    let mut reply = message::HyperionReply::new();
                    reply.set_field_type(message::HyperionReply_Type::REPLY);
                    reply.set_success(true);

                    reply
                })
                .forward(writer)
                .map(|_| {})
                .map_err(|e| {
                    warn!("error while processing request: {}", e);
                    ()
                });

            tokio::spawn(action);

            Ok(())
        });

    info!("server listening on {}", address);

    Ok(Box::new(server))
}

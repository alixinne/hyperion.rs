//! JSON protocol server implementation

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio_codec::Framed;

use serde_json::Value;

/// Schema definitions as Serde serializable structures and enums
mod message;

/// JSON protocol codec definition
mod codec;
use codec::*;

/// Binds the JSON protocol server implementation to the given address, returning a future to
/// process incoming requests.
///
/// # Parameters
///
/// * `address`: address (and port) of the endpoint to bind the JSON server to
///
/// # Errors
///
/// * When the server can't be bound to the given address
pub fn bind(
    address: &SocketAddr,
) -> Result<Box<dyn Future<Item = (), Error = std::io::Error> + Send>, failure::Error> {
    let listener = TcpListener::bind(&address)?;

    let server = listener.incoming().for_each(|socket| {
        debug!(
            "accepted new connection from {}",
            socket.peer_addr().unwrap()
        );

        let framed = Framed::new(socket, JsonCodec::new());
        let (writer, reader) = framed.split();

        let action = reader
            .and_then(|request| {
                debug!("processing request: {:?}", request);

                let mut reply = serde_json::Map::<_, _>::new();
                reply.insert("success".to_owned(), false.into());
                reply.insert("error".to_owned(), "not implemented".into());

                let reply = Value::from(reply);
                debug!("sending response: {:?}", reply);

                Ok(reply)
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

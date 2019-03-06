//! JSON protocol server implementation

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio_codec::Framed;

/// Module containing the schema definitions as Serde serializable structures and enums
mod message;

/// Parse an incoming request as JSON into the corresponding message type
///
/// # Parameters
///
/// * `line`: input request line to parse as a message
///
/// # Errors
///
/// When the line cannot be parsed as JSON, the underlying error is returned from serde_json.
fn parse_request(line: &str) -> serde_json::Result<message::HyperionMessage> {
    serde_json::from_str(line)
}

/// Encode an outgoing reply as JSON
///
/// # Parameters
///
/// * `reply`: reply to encode as JSON
///
/// # Errors
///
/// When the reply cannot be encoded as JSON, the underlying error is returned from serde_json.
fn encode_reply(reply: &serde_json::Value) -> serde_json::Result<String> {
    serde_json::to_string(reply)
}

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
pub fn bind(address: &SocketAddr) -> Result<Box<dyn Future<Item = (), Error = std::io::Error> + Send>, failure::Error> {
    let listener = TcpListener::bind(&address)?;

    let server = listener
        .incoming()
        .for_each(|socket| {
            debug!(
                "accepted new connection from {}",
                socket.peer_addr().unwrap()
            );

            let framed = Framed::new(socket, tokio_codec::LinesCodec::new());
            let (writer, reader) = framed.split();

            let action = reader
                .and_then(|request| {
                    debug!("got request: {}", request);
                    Ok(parse_request(&request)?)
                })
                .and_then(|request| {
                    debug!("processing request: {:?}", request);

                    let reply = serde_json::Value::String(String::from("ok"));

                    Ok(reply)
                })
                .and_then(|reply| {
                    debug!("sending reply: {}", reply);
                    Ok(encode_reply(&reply)?)
                })
                .forward(writer)
                .map(|_| { })
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

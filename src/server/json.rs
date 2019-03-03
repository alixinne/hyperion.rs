use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio_codec::Framed;

mod message;

fn parse_request(line: &str) -> serde_json::Result<message::HyperionMessage> {
    serde_json::from_str(line)
}

fn encode_reply(reply: &serde_json::Value) -> serde_json::Result<String> {
    serde_json::to_string(reply)
}

pub fn bind(address: &SocketAddr) -> Result<impl Future<Item = (), Error = ()>, failure::Error> {
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
        })
        .map_err(|err| {
            warn!("accept error: {:?}", err);
        });

    info!("server listening on {}", address);

    Ok(server)
}

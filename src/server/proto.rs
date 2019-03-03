use std::net::SocketAddr;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

use tokio_codec::{Decoder, Encoder, Framed};

use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};

use protobuf::Message;

mod message;

#[derive(Debug, Fail)]
enum HyperionMessageError {
    #[fail(display = "I/O error: {}", 0)]
    IoError(io::Error),
    #[fail(display = "decode error: {}", 0)]
    DecodeError(protobuf::error::ProtobufError),
    #[fail(display = "encode error: {}", 0)]
    EncodeError(protobuf::error::ProtobufError),
}

impl From<std::io::Error> for HyperionMessageError {
    fn from(error: std::io::Error) -> Self {
        HyperionMessageError::IoError(error)
    }
}

struct ProtoCodec {}

impl ProtoCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for ProtoCodec {
    type Item = message::HyperionRequest;
    type Error = HyperionMessageError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check that there is a size to be read
        if src.len() < 4 {
            return Ok(None);
        }

        let size = BigEndian::read_u32(&src[0..4]) as usize;

        // Check that we have the full message before decoding
        if src.len() - 4 < size {
            return Ok(None);
        }

        // Attempt to parse using protobuf
        let parsed = protobuf::parse_from_bytes::<Self::Item>(&src[4..(4 + size)]);

        // Consume the message from the buffer: since it's complete, the parsing
        // success does not depend on more data arriving
        src.advance(4 + size);

        // Process parse result
        match parsed {
            Ok(message) => Ok(Some(message)),
            Err(parse_error) => Err(HyperionMessageError::DecodeError(parse_error)),
        }
    }
}

impl Encoder for ProtoCodec {
    type Item = message::HyperionReply;
    type Error = HyperionMessageError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Get the size of the message
        let message_size = item.compute_size();

        // Reserve space in the dst buffer
        dst.reserve(4 + message_size as usize);

        // Write message size
        dst.put_u32_be(message_size as u32);

        // Write message contents
        item.write_to_writer(&mut dst.writer())
            .map_err(|e| HyperionMessageError::EncodeError(e))?;

        Ok(())
    }
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

//! JSON protocol server implementation

use std::convert::TryFrom;
use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

use crate::color;
use crate::hyperion::{Input, StateUpdate};
use crate::image::RawImage;
use crate::runtime::EffectDefinitionsHandle;

/// Schema definitions as Serde serializable structures and enums
mod message;
use message::{HyperionMessage, HyperionResponse};

/// JSON protocol codec definition
mod codec;
use codec::*;

pub use message::{Effect, EffectDefinition};

#[allow(missing_docs)]
mod errors {
    use error_chain::error_chain;

    error_chain! {
        types {
            JsonServerError, JsonServerErrorKind, ResultExt;
        }

        foreign_links {
            Io(::std::io::Error);
        }
    }
}

pub use errors::*;

/// Check if a message errors is a disconnect
fn is_disconnect(error: &HyperionMessageError) -> bool {
    if let HyperionMessageErrorKind::Io(io_error) = error.kind() {
        super::common::is_disconnect(io_error)
    } else {
        false
    }
}

async fn process(
    mut tx: mpsc::Sender<Input>,
    socket: TcpStream,
    peer_addr: SocketAddr,
    effect_definitions: EffectDefinitionsHandle,
) -> Result<(), mpsc::error::SendError<Input>> {
    let mut framed = Framed::new(socket, JsonCodec::default());
    let mut last_error = None;

    while let Some(request) = framed.next().await {
        trace!("processing request: {:?}", request);

        let reply = match request {
            Ok(HyperionMessage::ClearAll) => {
                // Update state
                tx.send(Input::user_input(StateUpdate::clear(), 0, None))
                    .await?;

                HyperionResponse::success()
            }
            Ok(HyperionMessage::Clear { priority }) => {
                // Update state
                tx.send(Input::user_input(StateUpdate::clear(), priority, None))
                    .await?;

                HyperionResponse::success()
            }
            Ok(HyperionMessage::Color {
                priority,
                duration,
                color,
            }) => {
                let update = StateUpdate::solid(color::ColorPoint::from((
                    f32::from(color[0]) / 255.0,
                    f32::from(color[1]) / 255.0,
                    f32::from(color[2]) / 255.0,
                )));

                // Update state
                tx.send(Input::user_input(update, priority, duration))
                    .await?;

                HyperionResponse::success()
            }
            Ok(HyperionMessage::Image {
                priority,
                duration,
                imagewidth,
                imageheight,
                imagedata,
            }) => {
                // Try to convert sizes to unsigned fields
                if let Ok(imagewidth) = u32::try_from(imagewidth) {
                    if let Ok(imageheight) = u32::try_from(imageheight) {
                        // Try to create image from raw data and given size
                        if let Ok(raw_image) =
                            RawImage::try_from((imagedata, imagewidth, imageheight))
                        {
                            // Update state
                            tx.send(Input::user_input(
                                StateUpdate::image(raw_image),
                                priority,
                                duration,
                            ))
                            .await?;

                            HyperionResponse::success()
                        } else {
                            HyperionResponse::error(&"invalid image data")
                        }
                    } else {
                        HyperionResponse::error(&"invalid image height")
                    }
                } else {
                    HyperionResponse::error(&"invalid image width")
                }
            }
            Ok(HyperionMessage::Effect {
                priority,
                duration,
                effect,
            }) => {
                // Update state
                tx.send(Input::effect(priority, duration, effect)).await?;

                // TODO: Only send success if effect was found
                HyperionResponse::success()
            }
            Ok(HyperionMessage::ServerInfo) => {
                let mut effects: Vec<_> = effect_definitions
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(_k, v)| v.get_definition().clone())
                    .collect();

                effects.sort_by(|a, b| a.name.cmp(&b.name));

                HyperionResponse::server_info(
                    hostname::get()
                        .map(|h| String::from(h.to_string_lossy()))
                        .unwrap_or_else(|_| "<unknown hostname>".to_owned()),
                    effects,
                    option_env!("HYPERION_VERSION_ID")
                        .unwrap_or("<unknown version>")
                        .to_owned(),
                )
            }
            Err(error) => {
                last_error = Some(error);
                HyperionResponse::error(last_error.as_ref().unwrap())
            }
            _ => HyperionResponse::error(&"not implemented"),
        };

        if last_error.is_some() {
            break;
        }

        trace!("sending response: {:?}", reply);

        if let Err(error) = framed.send(reply).await {
            last_error = Some(error);
            break;
        }
    }

    if let Some(error) = last_error {
        if !is_disconnect(&error) {
            warn!(
                "json({}): disconnecting client because of error: {:?}",
                peer_addr, error
            );
        }
    }

    info!("json({}): client disconnected", peer_addr);

    Ok(())
}

/// Binds the JSON protocol server implementation to the given address, returning a future to
/// process incoming requests.
///
/// # Parameters
///
/// * `address`: address (and port) of the endpoint to bind the JSON server to
/// * `effect_definitions`: handle to the effect definitions data
///
/// # Errors
///
/// * When the server can't be bound to the given address
pub async fn bind(
    address: impl tokio::net::ToSocketAddrs,
    effect_definitions: EffectDefinitionsHandle,
) -> Result<mpsc::Receiver<Input>, JsonServerError> {
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
                    tokio::spawn(process(
                        tx.clone(),
                        socket,
                        peer_addr,
                        effect_definitions.clone(),
                    ));
                }
                Err(error) => {
                    error!("accept error: {:?}", error);
                }
            }
        }
    });

    Ok(rx)
}

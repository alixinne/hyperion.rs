//! Definition of the UDP method

use std::time::{Duration, Instant};

use std::sync::{Arc, Mutex, MutexGuard};

use futures::{Future, Sink, Stream};

use websocket::client::r#async::Client;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};

use crate::methods::Method;
use crate::runtime::DeviceInstanceDataHandle;

#[allow(missing_docs)]
/// Internal error definitions
mod errors {
    use error_chain::error_chain;

    error_chain! {
        foreign_links {
            UrlParse(::websocket::client::builder::ParseError);
            WebSocket(::websocket::result::WebSocketError);
        }
    }
}

use errors::Error as WsError;

/// WebSocket client connect result
type WsClient = Client<Box<dyn websocket::stream::r#async::Stream + Send>>;
/// WebSocket write part
type WsWrite = futures::stream::SplitSink<
    tokio::codec::Framed<
        Box<dyn websocket::stream::r#async::Stream + Send>,
        websocket::codec::ws::MessageCodec<OwnedMessage>,
    >,
>;

/// State of the UDP socket in the async runtime
enum SocketState {
    /// Object was just initialized
    Initialized,
    /// Same as initialized, but when returned to from another state
    Errored {
        /// Time when a session allocation error has been signaled
        error_signaled: Instant,
    },
    /// Socket is connecting
    Connecting,
    /// The socket is ready for sending new data
    Ready {
        /// WebSocket write part
        write: WsWrite,
    },
    /// The socket is currently busy sending data
    Busy {
        /// Number of updates skipped while the socket was busy
        skipped_updates: usize,
        /// Data pending to be sent back as a pong
        pong_pending: Option<Vec<u8>>,
    },
}

impl Default for SocketState {
    fn default() -> Self {
        SocketState::Initialized
    }
}

/// Shared websocket state data
struct WsData {
    /// Source address of the target device
    address: String,
    /// Current session
    state: SocketState,
    /// Current device data to be sent
    current_data: Option<DeviceInstanceDataHandle>,
}

/// Handle to shared websocket state
type WsDataHandle = Arc<Mutex<WsData>>;

/// LED device that forwards RGB* data to a WebSocket
pub struct Ws {
    /// Handle to the shared state for this socket
    data: WsDataHandle,
}

impl Ws {
    /// Create a new websocket device method
    ///
    /// # Parameters
    ///
    /// * `address`: address of the target device
    pub fn new(address: String) -> Self {
        Self {
            data: Arc::new(Mutex::new(WsData {
                address,
                state: Default::default(),
                current_data: None,
            })),
        }
    }

    /// Start connecting the WebSocket
    ///
    /// # Parameters
    ///
    /// * `data`: locked state guard
    fn start_connect(&self, data: &mut MutexGuard<WsData>) -> Result<(), WsError> {
        let complete_data = self.data.clone();
        let error_data = self.data.clone();

        // We'll be busy connecting
        data.state = SocketState::Connecting;

        // Spawn future for connecting to device
        tokio::spawn(
            ClientBuilder::new(&data.address)?
                .async_connect(None)
                .map(move |result| Self::on_connect_complete(result, complete_data))
                .map_err(move |error| Self::on_connect_error(error, error_data)),
        );

        Ok(())
    }

    /// Start sending the given message over the WebSocket
    ///
    /// # Parameters
    ///
    /// * `write`: write part of the WebSocket
    /// * `device_data`: device instance to use for preparing the message
    /// * `data_handle`: handle to the shared state
    ///
    /// # Returns
    ///
    /// None if a message was queued, or Some(WsWrite) if no message is to be sent because the
    /// device is idle.
    #[must_use]
    fn start_send(
        write: WsWrite,
        device_data: DeviceInstanceDataHandle,
        data_handle: WsDataHandle,
    ) -> Option<WsWrite> {
        let complete_data = data_handle.clone();
        let error_data = data_handle.clone();

        if let Some(message) = device_data.write().unwrap().pass(|stats, leds| {
            use serde_json::{Number, Value};

            // Create list of LED data objects
            let mut led_objects: Vec<Value> = Vec::with_capacity(stats.led_count);

            for led in leds {
                // JSON object for current LED
                let mut led_map = serde_json::Map::with_capacity(stats.components);
                for (comp, ch) in led.formatted.into_iter() {
                    led_map.insert(
                        ch.to_string(),
                        Value::Number(
                            Number::from_f64(comp.into())
                                .unwrap_or_else(|| Number::from_f64(0.0f64).unwrap()),
                        ),
                    );
                }

                led_objects.push(Value::Object(led_map));
            }

            serde_json::to_string(&json!({ "leds": Value::Array(led_objects) }))
                .expect("failed to encode JSON message")
        }) {
            // Spawn future for writing to device
            tokio::spawn(
                write
                    .send(OwnedMessage::Text(message))
                    .map(move |result| Self::on_send_complete(result, complete_data))
                    .map_err(move |error| Self::on_send_error(error, error_data)),
            );

            None
        } else {
            Some(write)
        }
    }

    /// Start sending the given pong response over the WebSocket
    ///
    /// # Parameters
    ///
    /// * `write`: write part of the WebSocket
    /// * `pong_data`: data to send back as a pong
    /// * `data_handle`: handle to the shared state
    fn start_pong(write: WsWrite, pong_data: Vec<u8>, data_handle: WsDataHandle) {
        let complete_data = data_handle.clone();
        let error_data = data_handle.clone();

        // Spawn future for writing to device
        tokio::spawn(
            write
                .send(OwnedMessage::Pong(pong_data))
                .map(move |result| Self::on_send_complete(result, complete_data))
                .map_err(move |error| Self::on_send_error(error, error_data)),
        );
    }

    /// Try writing the new state to the WebSocket
    ///
    /// # Parameters
    ///
    /// * `device_data`: device instance to use for preparing the message
    fn try_write(&self, device_data: DeviceInstanceDataHandle) -> Result<(), WsError> {
        let mut data = self.data.lock().unwrap();
        data.current_data = Some(device_data);

        let old_state = std::mem::replace(&mut data.state, Default::default());
        match old_state {
            SocketState::Initialized => {
                self.start_connect(&mut data)?;
            }
            SocketState::Connecting => {
                // We're still connecting, we'll try next round
                data.state = old_state;
            }
            SocketState::Ready { write } => {
                data.state = SocketState::Busy {
                    skipped_updates: 0,
                    pong_pending: None,
                };

                if let Some(write) =
                    Self::start_send(write, data.current_data.take().unwrap(), self.data.clone())
                {
                    data.state = SocketState::Ready { write };
                }
            }
            SocketState::Busy {
                skipped_updates,
                pong_pending,
            } => {
                // Skip this update, we're still busy sending
                data.state = SocketState::Busy {
                    skipped_updates: skipped_updates + 1,
                    pong_pending,
                };
            }
            SocketState::Errored { error_signaled } => {
                if (Instant::now() - error_signaled) > Duration::from_millis(60000) {
                    trace!("ws({}): trying to connect again", &data.address);
                    self.start_connect(&mut data)?;
                } else {
                    data.state = SocketState::Errored { error_signaled }
                }
            }
        };

        Ok(())
    }

    /// Socket connect completion handler
    fn on_connect_complete(
        (socket, _headers): (WsClient, websocket::header::Headers),
        data: WsDataHandle,
    ) {
        let mut data_mut = data.lock().unwrap();
        let (write, read) = socket.split();

        let read_data = data.clone();

        // Start pumping messages out of read
        tokio::spawn(
            read.for_each(move |message| {
                let mut data = read_data.lock().unwrap();

                match message {
                    OwnedMessage::Ping(pong_data) => {
                        let old_state = std::mem::replace(&mut data.state, Default::default());
                        match old_state {
                            SocketState::Ready { write } => {
                                data.state = SocketState::Busy {
                                    skipped_updates: 0,
                                    pong_pending: None,
                                };
                                Self::start_pong(write, pong_data, read_data.clone());
                            }
                            SocketState::Busy {
                                skipped_updates, ..
                            } => {
                                data.state = SocketState::Busy {
                                    skipped_updates,
                                    pong_pending: Some(pong_data),
                                };
                            }
                            _ => {} // ignore other states
                        };
                    }
                    OwnedMessage::Text(text) => {
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(serde_json::Value::Object(map)) => {
                                if let Some(serde_json::Value::Bool(true)) = map.get("success") {
                                    // nothing to do, it's a success
                                } else if let Some(serde_json::Value::String(message)) =
                                    map.get("error")
                                {
                                    warn!(
                                        "ws({}): device returned error: {}",
                                        data.address, message
                                    );
                                } else {
                                    warn!(
                                        "ws({}): missing required fields in response: {}",
                                        data.address, text
                                    );
                                }
                            }
                            Ok(_other) => {
                                warn!(
                                    "ws({}): unexpected object type in response: {}",
                                    data.address, text
                                );
                            }
                            Err(error) => {
                                warn!("ws({}): failed to parse response: {}", data.address, error);
                            }
                        }
                    }
                    _ => {} // ignore other messages
                }

                Ok(())
            })
            .map_err(|_| ()),
        );

        // As soon as we're connected, we can send the pending message, or just
        // switch to the ready state
        if let Some(device_data) = data_mut.current_data.take() {
            if let Some(write) = Self::start_send(write, device_data, data.clone()) {
                data_mut.state = SocketState::Ready { write };
            }
        } else {
            data_mut.state = SocketState::Ready { write };
        }
    }

    /// Socket connect error handler
    fn on_connect_error(error: WebSocketError, data: WsDataHandle) {
        let mut data_mut = data.lock().unwrap();

        warn!(
            "ws({}): connect failed: {:?}",
            data_mut.address,
            WsError::from(error)
        );

        data_mut.state = SocketState::Errored {
            error_signaled: Instant::now(),
        };
    }

    /// Message send complete handler
    fn on_send_complete(write: WsWrite, data: WsDataHandle) {
        let mut data_mut = data.lock().unwrap();
        // TODO: do not clone just to log events
        let addr = data_mut.address.clone();

        // As soon as we've sent a message, check if we have more to send
        if let Some(device_data) = data_mut.current_data.take() {
            if let SocketState::Busy {
                skipped_updates,
                pong_pending,
            } = &mut data_mut.state
            {
                if let Some(pong_data) = pong_pending.take() {
                    // We're sending a pong answer, so keep the send request for later
                    data_mut.current_data = Some(device_data);

                    Self::start_pong(write, pong_data, data.clone());
                    return;
                }

                if *skipped_updates > 0 {
                    trace!("ws({}): skipped {} updates", addr, skipped_updates);
                }
            }

            data_mut.state = SocketState::Busy {
                skipped_updates: 0,
                pong_pending: None,
            };

            if let Some(write) = Self::start_send(write, device_data, data.clone()) {
                data_mut.state = SocketState::Ready { write };
            }
        } else {
            data_mut.state = SocketState::Ready { write };
        }
    }

    /// Message send error handler
    fn on_send_error(error: WebSocketError, data: WsDataHandle) {
        let mut data_mut = data.lock().unwrap();

        warn!(
            "ws({}): send failed: {:?}",
            data_mut.address,
            WsError::from(error)
        );

        data_mut.state = SocketState::Errored {
            error_signaled: Instant::now(),
        };
    }
}

impl Method for Ws {
    fn write(&mut self, data: DeviceInstanceDataHandle) {
        if let Err(error) = self.try_write(data) {
            let mut data = self.data.lock().unwrap();
            warn!("ws({}): failed: {}", data.address, error);

            data.state = SocketState::Errored {
                error_signaled: Instant::now(),
            };
        }
    }
}

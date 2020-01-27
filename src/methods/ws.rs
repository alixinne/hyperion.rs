//! Definition of the WS method

use std::sync::{Arc, Mutex};

use futures_util::SinkExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{self, Message};

use super::{Method, WriteError, WriteResult};
use crate::runtime::LedData;

use serde_json::json;

type WebSocket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;
type ConnectResult =
    Result<(WebSocket, tungstenite::handshake::client::Response), tungstenite::error::Error>;

/// WS session data
enum Session {
    Initialized,
    Resolving,
    Bound {
        /// WS socket to the device
        socket: WebSocket,
    },
    Errored {
        error: String,
    },
    Pending,
}

impl Session {
    fn new() -> Self {
        Self::Initialized
    }

    async fn do_bind(address: Arc<url::Url>) -> ConnectResult {
        connect_async((*address).clone()).await
    }

    /// Connect to the websocket target
    ///
    /// # Parameters
    ///
    /// * `address`: remote address to connect to
    async fn bind(
        this: SessionState,
        address: Arc<url::Url>,
    ) -> Result<(), tungstenite::error::Error> {
        *this.lock().unwrap() = Self::Resolving;

        match Self::do_bind(address).await {
            Ok((socket, _response)) => {
                *this.lock().unwrap() = Session::Bound { socket };

                Ok(())
            }
            Err(error) => {
                *this.lock().unwrap() = Session::Errored {
                    error: error.to_string(),
                };

                Err(error)
            }
        }
    }
}

/// Handle to the WS session
type SessionState = Arc<Mutex<Session>>;

/// LED device that forwards raw RGB data as WS packets
pub struct Ws {
    /// Source address of the target device
    address: Arc<url::Url>,
    /// Current session
    session: SessionState,
}

impl Ws {
    /// Create a new WS device method
    ///
    /// # Parameters
    ///
    /// * `address`: address and port of the target device
    pub fn new(address: url::Url) -> Self {
        let address = Arc::new(address);

        // Create Ws method object
        let mut this = Self {
            address,
            session: Arc::new(Mutex::new(Session::new())),
        };

        // Start resolving and binding as soon as possible
        this.start_resolving();

        this
    }

    fn start_resolving(&mut self) {
        // Start resolving the remote address
        tokio::spawn(Session::bind(self.session.clone(), self.address.clone()));
    }

    /// Send a frame on the target WS socket
    ///
    /// # Parameters
    ///
    /// * `socket`: WS socket object
    /// * `data`: device instance to use for preparing the datagram
    async fn send_data(
        socket: &mut WebSocket,
        data: &Vec<LedData>,
    ) -> Result<(), tungstenite::error::Error> {
        use serde_json::{Number, Value};

        // Create list of LED data objects
        let mut led_objects: Vec<Value> = Vec::with_capacity(data.len());
        let components = data[0].formatted.components();

        for led in data {
            // JSON object for current LED
            let mut led_map = serde_json::Map::with_capacity(components);
            for (comp, ch) in led.formatted.iter() {
                led_map.insert(
                    ch.to_string(),
                    Value::Number(
                        Number::from_f64((*comp).into())
                            .unwrap_or_else(|| Number::from_f64(0.0f64).unwrap()),
                    ),
                );
            }

            led_objects.push(Value::Object(led_map));
        }

        let message = Message::Text(
            serde_json::to_string(&json!({ "leds": Value::Array(led_objects) }))
                .expect("failed to encode JSON message"),
        );

        socket.send(message).await
    }
}

#[async_trait]
impl Method for Ws {
    async fn write(&mut self, led_data: &Vec<LedData>) -> WriteResult {
        let mut s = None;

        // Block where the session mutex is locked
        {
            let mut session = self.session.lock().unwrap();
            let old_state = std::mem::replace(&mut *session, Session::Pending);

            let result = match old_state {
                Session::Initialized => {
                    // Start resolving async
                    *session = Session::Initialized;
                    drop(session);
                    self.start_resolving();
                    Some(Err(WriteError::NotReady))
                }
                Session::Resolving => {
                    // We're still resolving
                    *session = Session::Resolving;
                    Some(Err(WriteError::NotReady))
                }
                Session::Errored { error } => {
                    // We failed to resolve, try again later
                    *session = Session::Initialized;
                    Some(Err(WriteError::Errored { error }))
                }
                Session::Bound { socket } => {
                    // The socket is bound, write to it
                    s = Some(socket);
                    None
                }
                Session::Pending => panic!("encountered pending ws session"),
            };

            if let Some(res) = result {
                // Return result if we failed
                return res;
            }
        }

        // If we get to that point, everything is ready for writing
        match Ws::send_data(s.as_mut().unwrap(), led_data).await {
            Ok(_written) => {
                *self.session.lock().unwrap() = Session::Bound { socket: s.unwrap() };

                Ok(())
            }
            Err(error) => {
                error!(
                    "ws({}): sending message failed: {:?}",
                    self.address.as_str(),
                    error
                );

                // Reinitialize session, next calls to write will re-allocate the socket
                *self.session.lock().unwrap() = Session::new();
                Err(WriteError::NotReady)
            }
        }
    }
}

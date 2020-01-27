//! Definition of the UDP method

use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, ToSocketAddrs};

use std::sync::{Arc, Mutex};

use tokio::net::UdpSocket;

use super::{Method, WriteError, WriteResult};
use crate::runtime::LedData;

/// UDP session data
#[derive(Debug)]
enum Session {
    Initialized,
    Resolving,
    Bound {
        /// Address of the target device
        remote_addr: SocketAddr,
        /// UDP socket to the device
        socket: UdpSocket,
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

    async fn do_resolve(address: Arc<String>) -> Result<(SocketAddr, UdpSocket)> {
        // Resolve remote addr
        let remote_addr = tokio::task::block_in_place(|| address.to_socket_addrs())?
            .next()
            .ok_or_else(|| Error::from(ErrorKind::NotFound))?;

        // Choose correct IP version for local addr
        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()
        .unwrap();

        // Bind socket to local addr
        let socket = UdpSocket::bind(&local_addr).await?;

        Ok((remote_addr, socket))
    }

    /// Resolve the remote address and bind an UDP socket
    ///
    /// # Parameters
    ///
    /// * `address`: remote address to resolve
    async fn resolve(this: SessionState, address: Arc<String>) -> Result<()> {
        *this.lock().unwrap() = Self::Resolving;

        match Self::do_resolve(address).await {
            Ok((remote_addr, socket)) => {
                *this.lock().unwrap() = Session::Bound {
                    remote_addr,
                    socket,
                };

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

/// Handle to the UDP session
type SessionState = Arc<Mutex<Session>>;

/// LED device that forwards raw RGB data as UDP packets
pub struct Udp {
    /// Source address of the target device
    address: Arc<String>,
    /// Current session
    session: SessionState,
    /// Buffer for UDP packets
    buffer: Vec<u8>,
}

impl Udp {
    /// Create a new UDP device method
    ///
    /// # Parameters
    ///
    /// * `address`: address and port of the target device
    pub fn new(address: String) -> Self {
        let address = Arc::new(address);

        // Create Udp method object
        let mut this = Self {
            address,
            session: Arc::new(Mutex::new(Session::new())),
            buffer: Vec::new(),
        };

        // Start resolving and binding as soon as possible
        this.start_resolving();

        this
    }

    fn start_resolving(&mut self) {
        // Start resolving the remote address
        tokio::spawn(Session::resolve(self.session.clone(), self.address.clone()));
    }

    /// Send a frame on the target UDP socket
    ///
    /// # Parameters
    ///
    /// * `buffer`: reusable buffer
    /// * `socket`: UDP socket object
    /// * `data`: device instance to use for preparing the datagram
    /// * `remote_addr': target address of the device
    async fn send_data(
        buffer: &mut Vec<u8>,
        socket: &mut UdpSocket,
        data: &Vec<LedData>,
        remote_addr: &SocketAddr,
    ) -> Result<usize> {
        // Create buffer with correct size
        let components = data[0].formatted.components();
        buffer.resize(data.len() * components, 0);

        // Fill buffer with data
        for led in data {
            for (idx, (comp, _ch)) in led.formatted.iter().enumerate() {
                buffer[led.index * components + idx] = (comp * 255.0f32) as u8;
            }
        }

        socket.send_to(&buffer[..], &remote_addr).await
    }
}

#[async_trait]
impl Method for Udp {
    async fn write(&mut self, led_data: &Vec<LedData>) -> WriteResult {
        let (mut ra, mut s) = (None, None);

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
                Session::Bound {
                    remote_addr,
                    socket,
                } => {
                    // The socket is bound, write to it
                    ra = Some(remote_addr);
                    s = Some(socket);
                    None
                }
                Session::Pending => panic!("encountered pending udp session"),
            };

            if let Some(res) = result {
                // Return result if we failed
                return res;
            }
        }

        // If we get to that point, everything is ready for writing
        match Udp::send_data(
            &mut self.buffer,
            s.as_mut().unwrap(),
            led_data,
            ra.as_ref().unwrap(),
        )
        .await
        {
            Ok(_written) => {
                *self.session.lock().unwrap() = Session::Bound {
                    remote_addr: ra.unwrap(),
                    socket: s.unwrap(),
                };

                Ok(())
            }
            Err(error) => {
                error!(
                    "udp({}): sending datagram failed: {:?}",
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

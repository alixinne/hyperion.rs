//! Definition of the UDP method

use std::time::{Duration, Instant};

use std::mem::replace;

use std::io::{Error, ErrorKind, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

use std::sync::{Arc, Mutex, MutexGuard};

use tokio::net::udp::UdpSocket;

use futures::Future;

use crate::config::ColorFormat;
use crate::filters::ColorFilter;
use crate::methods::Method;
use crate::runtime::{IdleTracker, LedInstance};

/// State of the UDP socket in the async runtime
#[derive(Debug)]
enum SocketState {
    /// Empty state
    Idle {
        /// Time when a session allocation error has been signalled
        error_signaled: Option<Instant>,
    },
    /// The socket is ready for sending new data
    Ready(UdpSocket),
    /// The socket is currently busy sending data
    Busy,
    /// The socket is busy sending data, and new data is available
    /// Tracks the number of discarded updates.
    Pending(Vec<u8>, usize),
}

impl Default for SocketState {
    fn default() -> Self {
        SocketState::Idle {
            error_signaled: None,
        }
    }
}

/// UDP session data
struct Session {
    /// Address of the target device
    remote_addr: SocketAddr,
    /// State of the UDP socket to the device
    state: SocketState,
}

impl Session {
    /// Resolve the remote address and bind an UDP socket
    fn new(address: &str) -> Result<Self> {
        // Resolve remote addr
        let remote_addr = address
            .to_socket_addrs()?
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

        Ok(Self {
            remote_addr,
            state: SocketState::Ready(UdpSocket::bind(&local_addr)?),
        })
    }

    /// Get a new unbound session
    fn empty() -> Self {
        Self {
            remote_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1000),
            state: SocketState::default(),
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
    /// UDP packet buffer A
    rgb_buffer_a: Vec<u8>,
    /// UDP packet buffer B
    rgb_buffer_b: Vec<u8>,
}

impl Udp {
    /// Create a new UDP device method
    ///
    /// # Parameters
    ///
    /// * `address`: address and port of the target device
    pub fn new(address: String) -> Result<Self> {
        Ok(Self {
            address: Arc::new(address),
            session: Arc::new(Mutex::new(Session::empty())),
            rgb_buffer_a: Vec::new(),
            rgb_buffer_b: Vec::new(),
        })
    }

    /// Fill the target buffer with device LED data
    fn fill_buffer(
        rgb_buffer: &mut Vec<u8>,
        time: Instant,
        filter: &ColorFilter,
        leds: &mut [LedInstance],
        idle_tracker: &mut IdleTracker,
        format: &ColorFormat,
    ) {
        // Number of components per LED
        let components = format.components();

        // Set correct buffer size
        rgb_buffer.resize(leds.len() * components, 0);

        // Fill buffer with data
        for (i, led) in leds.iter_mut().enumerate() {
            let current_color = led.next_value(time, &filter, idle_tracker);
            let device_color = current_color.to_device(format);
            let formatted = device_color.format(format);

            for (idx, comp) in formatted.into_iter().enumerate() {
                rgb_buffer[i * components + idx] = (comp * 255.0f32) as u8;
            }
        }
    }

    /// State updater for send_dgram completions
    fn next_dgram(
        (socket, _buffer): (UdpSocket, Vec<u8>),
        address: Arc<String>,
        session_ref: SessionState,
    ) {
        // TODO: return buffer to the object

        let mut session = session_ref.lock().unwrap();
        let old_state = replace(&mut session.state, SocketState::default());

        session.state = match old_state {
            SocketState::Busy => SocketState::Ready(socket),
            SocketState::Pending(buffer, skipped_updates) => {
                if skipped_updates > 0 {
                    trace!(
                        "udp({}): skipped {} updates",
                        address.as_str(),
                        skipped_updates
                    );
                }

                Self::send_buffer(socket, buffer, address, &session, session_ref.clone());

                SocketState::Busy
            }
            other => other,
        }
    }

    /// Completion handler for the UDP futures
    fn on_send_dgram_complete(
        (socket, buffer): (UdpSocket, Vec<u8>),
        address: Arc<String>,
        session_ref: SessionState,
    ) {
        Self::next_dgram((socket, buffer), address, session_ref);
    }

    /// Error handler for the UDP futures
    fn on_send_dgram_error(
        error: tokio::io::Error,
        address: Arc<String>,
        session_ref: SessionState,
    ) {
        error!(
            "udp({}): sending datagram failed: {:?}",
            address.as_str(),
            error
        );

        // Reinitialize state, next calls to write will re-allocate the socket
        *session_ref.lock().unwrap() = Session::empty();
    }

    /// Send a byte buffer on the target UDP socket
    ///
    /// # Parameters
    ///
    /// * `socket`: UDP socket object
    /// * `buffer`: datagram to send
    /// * `address': source address of the device
    /// * `session_locked`: reference to the locked session (prevents recursive locking of `session`)
    /// * `session`: reference to the UDP session
    fn send_buffer(
        socket: UdpSocket,
        buffer: Vec<u8>,
        address: Arc<String>,
        session_locked: &MutexGuard<Session>,
        session: SessionState,
    ) {
        let session_ref = Arc::clone(&session);
        let remote_addr = session_locked.remote_addr;
        let address_ref = address.clone();

        tokio::spawn(
            socket
                .send_dgram(buffer, &remote_addr)
                .map(move |result| Self::on_send_dgram_complete(result, address, session))
                .map_err(move |error| Self::on_send_dgram_error(error, address_ref, session_ref)),
        );
    }
}

impl Method for Udp {
    fn write(
        &mut self,
        time: Instant,
        filter: &ColorFilter,
        leds: &mut [LedInstance],
        idle_tracker: &mut IdleTracker,
        format: &ColorFormat,
    ) {
        let mut session = self.session.lock().unwrap();
        let old_state = replace(&mut session.state, SocketState::default());

        // We need to re-allocate the session, if it failed previously
        let old_state = match old_state {
            SocketState::Idle { error_signaled } => {
                if error_signaled.is_none()
                    || (Instant::now() - error_signaled.unwrap()) > Duration::from_millis(60000)
                {
                    trace!("udp({}): trying new session", self.address.as_str());
                    let new_session = Session::new(&self.address);

                    match new_session {
                        Ok(mut new_session) => {
                            session.remote_addr = new_session.remote_addr;
                            trace!(
                                "udp({}): success: {}",
                                self.address.as_str(),
                                new_session.remote_addr
                            );
                            replace(&mut new_session.state, SocketState::default())
                        }
                        Err(error) => {
                            if error_signaled.is_none() {
                                warn!("udp({}): failed to bind socket: {:?}", &self.address, error);
                            }

                            trace!("udp({}): failed: {:?}", self.address.as_str(), error);

                            SocketState::Idle {
                                error_signaled: Some(Instant::now()),
                            }
                        }
                    }
                } else {
                    SocketState::Idle { error_signaled }
                }
            }
            other => other,
        };

        // Now try to send the packet
        session.state = match old_state {
            SocketState::Idle { error_signaled } => SocketState::Idle { error_signaled },
            SocketState::Ready(socket) => {
                // Socket ready for sending, prepare a buffer and start sending it

                let mut buffer = replace(&mut self.rgb_buffer_a, Vec::new());

                Self::fill_buffer(&mut buffer, time, filter, leds, idle_tracker, format);

                Self::send_buffer(
                    socket,
                    buffer,
                    self.address.clone(),
                    &session,
                    self.session.clone(),
                );

                SocketState::Busy
            }
            SocketState::Busy => {
                // Socket busy, prepare a secondary buffer to send when the socket
                // is ready again

                let mut buffer = replace(&mut self.rgb_buffer_b, Vec::new());

                Self::fill_buffer(&mut buffer, time, filter, leds, idle_tracker, format);

                SocketState::Pending(buffer, 0)
            }
            SocketState::Pending(mut buffer, skipped_updates) => {
                // Socket busy, replace the existing secondary buffer with up-to-date
                // data to send when the socket is ready again

                Self::fill_buffer(&mut buffer, time, filter, leds, idle_tracker, format);

                SocketState::Pending(buffer, skipped_updates + 1)
            }
        };
    }
}

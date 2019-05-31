use std::time::Instant;

use std::cell::RefCell;

use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use super::{LedInstance, Method};

use crate::filters::ColorFilter;
use crate::runtime::IdleTracker;

/// LED device that forwards raw RGB data as UDP packets
pub struct Udp {
    remote_addr: SocketAddr,
    socket: UdpSocket,
    rgb_buffer: RefCell<Vec<u8>>,
}

impl Udp {
    pub fn new(address: String) -> Result<Self> {
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
            socket: UdpSocket::bind(&local_addr)?,
            rgb_buffer: RefCell::new(Vec::new()),
        })
    }
}

impl Method for Udp {
    fn write(&self, time: Instant, filter: &ColorFilter, leds: &mut [LedInstance], idle_tracker: &mut IdleTracker) {
        // Get reference to buffer for UDP data
        let mut rgb_buffer = self.rgb_buffer.borrow_mut();

        // Set correct buffer size
        rgb_buffer.resize(leds.len() * 3usize, 0);

        // Fill buffer with data
        for (i, led) in leds.iter_mut().enumerate() {
            let current_color = led.next_value(time, &filter, idle_tracker);
            let (r, g, b) = current_color.into_components();

            rgb_buffer[i * 3] = (r * 255.0f32) as u8;
            rgb_buffer[i * 3 + 1] = (g * 255.0f32) as u8;
            rgb_buffer[i * 3 + 2] = (b * 255.0f32) as u8;
        }

        self.socket
            .send_to(&rgb_buffer[..], &self.remote_addr)
            .expect("failed to send data");
    }
}

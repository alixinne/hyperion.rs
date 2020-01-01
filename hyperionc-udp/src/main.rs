//! `hyperionc-udp` is an implementation of the raw UDP protocol for debugging.
//!
//! Values received through the bound socket are forwarded to the standard
//! output in CSV format (tab separated, for gnuplot).
//!
//! # Usage
//!
//!    $ cargo run -- --help
//!     hyperionc-udp 0.1.0
//!     
//!     USAGE:
//!         hyperionc-udp [OPTIONS]
//!     
//!     FLAGS:
//!         -h, --help       Prints help information
//!         -V, --version    Prints version information
//!     
//!     OPTIONS:
//!         -b, --bind <bind>                Address to bind to [default: 0.0.0.0:19446]
//!         -c, --components <components>    Number of components per LED [default: 3]
//!         -l, --count <led-count>          Number of LEDs [default: 1]
//!
//!
//! # Authors
//!
//! * [Vincent Tavernier](https://github.com/vtavernier)
//!
//! # License
//!
//! This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Address to bind to
    #[structopt(short, long, default_value = "0.0.0.0:19446")]
    bind: SocketAddr,

    /// Number of LEDs
    #[structopt(short = "l", long = "count", default_value = "1")]
    led_count: usize,

    /// Number of components per LED
    #[structopt(short = "c", long = "components", default_value = "3")]
    components: usize,
}

fn led_name(led: usize, total_leds: usize) -> String {
    if total_leds > 1 {
        "L".to_owned() + &led.to_string()
    } else {
        "".to_owned()
    }
}

fn component_name(component: usize, total_components: usize) -> String {
    match component {
        0 => "R".to_owned(),
        1 => "G".to_owned(),
        2 => "B".to_owned(),
        3 => (if total_components > 4 { "C" } else { "W" }).to_owned(),
        4 => "W".to_owned(),
        _ => component.to_string(),
    }
}

fn lc_name(led_idx: usize, component: usize, opt: &Opt) -> String {
    led_name(led_idx, opt.led_count) + &component_name(component, opt.components)
}

#[paw::main]
fn main(opt: Opt) -> Result<()> {
    // Bind socket
    let socket = UdpSocket::bind(opt.bind)?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    // CSV writer object
    let mut csv = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(std::io::stdout());

    // Build header
    let mut header = vec!["time".to_owned(), "source".to_owned()];
    header.reserve(opt.led_count * opt.components);

    for led_idx in 0..opt.led_count {
        for component in 0..opt.components {
            header.push(lc_name(led_idx, component, &opt));
        }
    }

    csv.write_record(&header)?;
    csv.flush()?;

    // Vector of LED values
    let mut led_data = vec![vec![0u8; opt.components]; opt.led_count];

    // Configure ctrlc handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("failed to set ctrlc handler");

    // Buffer for incoming packets
    let mut buf = vec![0u8; opt.led_count * opt.components];

    // Start point
    let mut start = None;

    while running.load(Ordering::SeqCst) {
        if let Ok((read, remote_addr)) = socket.recv_from(&mut buf[..]) {
            if read > opt.led_count * opt.components {
                eprintln!(
                    "{}: too much data ({}, expected max. {})",
                    remote_addr,
                    read,
                    opt.led_count * opt.components
                );
            } else if read % opt.led_count != 0 {
                eprintln!(
                    "{}: not enough data for all LEDs (extra {})",
                    remote_addr,
                    read % opt.led_count
                );
            } else {
                // Number of available components per LED
                let read_components = read / opt.led_count;

                // Read available components
                for led_idx in 0..opt.led_count {
                    for component in 0..opt.components {
                        led_data[led_idx][component] = if component < read_components {
                            buf[led_idx * read_components + component]
                        } else {
                            // Extra components are set to 0
                            0
                        };
                    }
                }

                // Write record
                if start.is_none() {
                    start = Some(Instant::now());
                }

                csv.write_field(start.unwrap().elapsed().as_secs_f64().to_string())?;
                csv.write_field(remote_addr.to_string())?;
                for led_idx in 0..opt.led_count {
                    for component in 0..opt.components {
                        csv.write_field(led_data[led_idx][component].to_string())?;
                    }
                }
                csv.write_record(None::<&[u8]>)?;
                csv.flush()?;
            }
        }
    }

    Ok(())
}

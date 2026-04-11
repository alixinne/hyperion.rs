use std::io::Write;
use std::net::TcpStream;
use std::os::linux::net::TcpStreamExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use clap::Parser;
use hyperion::api::proto::message::ClearRequest;
use hyperion::api::proto::message::HyperionRequest;
use hyperion::api::proto::message::ImageRequest;
use hyperion::servers::proto::ProtoCodec;
use tokio_util::codec::Encoder;
use tracing::error;

mod capture;

const HYPERION_PRIORITY: i32 = 100;

#[derive(clap::Parser)]
#[command(version, about)]
struct Opts {
    /// Path to the DRI card device node to use
    #[arg(long)]
    card: PathBuf,

    /// Offset for the linear tone mapping curve
    #[arg(long, default_value = "0.0")]
    tone_mapping_offset: f32,

    /// Scaling factor for the linear tone mapping curve
    #[arg(long, default_value = "1.0")]
    tone_mapping_scaling: f32,

    /// Target host (hyperion, protobuf server)
    #[arg(long)]
    target_host: String,

    /// Buffer image width
    #[arg(long, default_value = "180")]
    image_width: u32,

    /// Log verbosity
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    /// Capture FPS
    #[arg(long, default_value = "30")]
    fps: u32,
}

struct ImageMessage {
    buffer: Bytes,
    width: u32,
    height: u32,
}

fn network_loop(
    term: Arc<AtomicBool>,
    rx: crossbeam_channel::Receiver<ImageMessage>,
    target_host: String,
    fps: u32,
) -> color_eyre::eyre::Result<()> {
    // Connect to the lighting server
    let mut conn = TcpStream::connect(&target_host)?;
    conn.set_nodelay(true)?;
    conn.set_quickack(true)?;

    // Hyperion protobuf codec
    let mut proto_codec = ProtoCodec::new();

    // Working buffer for RGBA -> RGB conversion
    let mut rgb_buf: Vec<u8> = Vec::new();

    // Reusable buffer for packet encodings
    let mut pkt_buf = BytesMut::new();

    while !term.load(Ordering::Relaxed) {
        if let Ok(msg) = rx.recv_timeout(Duration::from_micros(1_000_000 / ((fps as u64) * 2))) {
            // Hyperion expects RGB, not RGBA
            rgb_buf.clear();
            rgb_buf.reserve((msg.width * msg.height * 3) as _);
            for chunk in msg.buffer.chunks(4) {
                rgb_buf.extend(&chunk[0..3]);
            }

            // Encode and send request
            let mut req = HyperionRequest::default();
            req.set_command(hyperion::api::proto::message::hyperion_request::Command::Image);
            req.image_request = Some(ImageRequest {
                priority: HYPERION_PRIORITY,
                imagewidth: msg.width as _,
                imageheight: msg.height as _,
                imagedata: rgb_buf.clone(),
                duration: Some(1000),
            });

            pkt_buf.clear();
            proto_codec.encode(req, &mut pkt_buf)?;
            conn.write_all(&pkt_buf)?;
        }
    }

    // Encode and send request to close session
    let mut req = HyperionRequest::default();
    req.set_command(hyperion::api::proto::message::hyperion_request::Command::Clear);
    req.clear_request = Some(ClearRequest { priority: 100 });

    Ok(())
}

fn main() -> color_eyre::eyre::Result<()> {
    // Parse CLI options
    let opts = Opts::parse();

    color_eyre::install()?;
    install_tracing(&opts)?;

    // Initialize capture: this must happen on the main thread
    let mut capture = capture::init(opts.card.as_path(), opts.image_width)?;

    // Handle sigterm/sigint
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

    // Start network loop
    let (tx, rx) = crossbeam_channel::bounded(1);
    let jh = std::thread::spawn({
        let term = term.clone();
        let target_host = opts.target_host.clone();
        move || {
            while !term.load(Ordering::Relaxed) {
                if let Err(error) =
                    network_loop(term.clone(), rx.clone(), target_host.clone(), opts.fps)
                {
                    error!(%error, "Network loop failed, restarting in 5s");
                    sleep(Duration::from_secs(5));
                }
            }
        }
    });

    // Frame capture loop
    let mut frame_buf = BytesMut::new();
    while !term.load(Ordering::Relaxed) {
        let start_frame = Instant::now();
        let next_frame = start_frame + Duration::from_micros(1_000_000 / opts.fps as u64);

        // Capture frame
        let (width, height) = capture.next_frame(
            &mut frame_buf,
            opts.tone_mapping_offset,
            opts.tone_mapping_scaling,
        );

        // Forward it to network loop, but if the network loop is busy just drop it and move on to
        // the next frame to avoid running late
        let _ = tx.send_deadline(
            ImageMessage {
                buffer: frame_buf.copy_to_bytes(frame_buf.len()),
                width,
                height,
            },
            next_frame,
        );

        // Wait for next frame
        let now = Instant::now();
        if next_frame > now {
            sleep(next_frame - now);
        }
    }

    jh.join().expect("join network loop thread failed");

    Ok(())
}

fn install_tracing(opts: &Opts) -> Result<(), tracing_subscriber::util::TryInitError> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{fmt, prelude::*};

    let fmt_layer = fmt::layer();

    tracing_subscriber::registry()
        .with(opts.verbosity.tracing_level_filter())
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .try_init()
}

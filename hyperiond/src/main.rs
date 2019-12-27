//! `hyperiond` is the Rust implementation of the
//! [Hyperion](https://github.com/hyperion-project/hyperion) ambient lighting software. It is
//! written from scratch both as an experiment and as a way to add more features.
//!
//! # Usage
//!
//! For now, the CLI is only able to start the hyperion server implementation:
//!
//!     $ cargo run -- server --help
//!     hyperiond 0.2.0
//!     
//!     USAGE:
//!         hyperiond [OPTIONS]
//!     
//!     FLAGS:
//!         -h, --help       Prints help information
//!         -V, --version    Prints version information
//!     
//!     OPTIONS:
//!         -b, --bind <bind>                IP address to bind the servers to [default: 127.0.0.1]
//!         -c, --config <config>            Path to the configuration file [default: config.yml]
//!         -j, --json-port <json-port>      TCP port for the JSON server [default: 19444]
//!         -p, --proto-port <proto-port>    TCP port for the Protobuf server [default: 19445]
//!         -w, --web-port <web-port>        TCP port for the Web interface
//!
//! Logging is set using the HYPERION_LOG environment variable, which can be set to the desired
//! logging level (trace, debug, info, warn, error). Note that this will affect logging of all
//! crates, and if only hyperion logging is required, it should be filtered as such:
//! `HYPERION_LOG=hyperion=level`. See the [env_logger crate docs](https://docs.rs/env_logger/0.6.1/env_logger/)
//! for more details.
//!
//! # Development
//!
//! The source code in this folder is only responsible for the command-line interface and starting
//! the server code, which is implemented in the [core crate](../hyperion)
//!
//! # Authors
//!
//! * [Vincent Tavernier](https://github.com/vtavernier)
//!
//! # License
//!
//! This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
use std::path::PathBuf;
use std::str::FromStr;

use env_logger::Env;
use error_chain::error_chain;
use structopt::StructOpt;

error_chain! {
    types {
        HyperionError, HyperionErrorKind, ResultExt;
    }

    links {
        Config(hyperion::config::ConfigError, hyperion::config::ConfigErrorKind);
        Hyperion(hyperion::hyperion::ServiceError, hyperion::hyperion::ServiceErrorKind);
    }

    errors {
        InvalidBindAddress(t: String) {
            description("invalid bind address")
            display("failed to resolve bind address: '{}'", t)
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to the configuration file
    #[structopt(short, long, parse(from_os_str), default_value = "config.yml")]
    config: PathBuf,

    /// IP address to bind the servers to
    #[structopt(short, long, default_value = "127.0.0.1")]
    bind: String,

    /// TCP port for the JSON server
    #[structopt(short, long, default_value = "19444")]
    json_port: u16,

    /// TCP port for the Protobuf server
    #[structopt(short, long, default_value = "19445")]
    proto_port: u16,

    /// TCP port for the Web interface
    #[structopt(short, long)] // default 19080
    web_port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), HyperionError> {
    // Load options
    let opt = Opt::from_args();

    // Initialize logging, default to info
    env_logger::from_env(Env::default().filter_or("HYPERION_LOG", "hyperion=info")).init();

    // Resolve address
    let bind_addr = std::net::IpAddr::from_str(&opt.bind)
        .chain_err(|| HyperionErrorKind::InvalidBindAddress(opt.bind.clone()))?;

    // Parse config file
    let config = hyperion::config::Config::read(opt.config)?;

    // No web support for now
    if opt.web_port.is_some() {
        panic!("The web UI is not supported currently. Please use without --web-port");
    }

    // Run server
    Ok(hyperion::hyperion::service::run(
        (bind_addr, opt.json_port),
        (bind_addr, opt.proto_port),
        config,
    )
    .await?)
}

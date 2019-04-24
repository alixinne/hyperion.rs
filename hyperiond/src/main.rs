//! `hyperiond` is the Rust implementation of the
//! [Hyperion](https://github.com/hyperion-project/hyperion) ambient lighting software. It is
//! written from scratch both as an experiment and as a way to add more features.
//!
//! # Usage
//!
//! For now, the CLI is only able to start the hyperion server implementation:
//!
//!     $ cargo run -- server --help
//!     hyperiond-server 0.1.0
//!     Starts the server daemon
//!     
//!     USAGE:
//!         hyperiond --config <FILE> server [OPTIONS] --bind <ADDRESS> --json-port <PORT> --proto-port <PORT>
//!     
//!     FLAGS:
//!         -h, --help    Prints help information
//!     
//!     OPTIONS:
//!             --bind <ADDRESS>             IP address to bind the servers to [default: 127.0.0.1]
//!             --disable-devices <REGEX>    Disable matching devices from updates
//!             --json-port <PORT>           TCP port for the JSON server [default: 19444]
//!             --proto-port <PORT>          TCP port for the Protobuf server [default: 19445]
//!
//! Logging is set using the HYPERION_LOG environment variable, which can be set to the desired
//! logging level (trace, debug, info, warn, error). Note that this will affect logging of all
//! crates, and if only hyperion logging is required, it should be filtered as such:
//! `HYPERION_LOG=hyperion=level`. See https://docs.rs/env_logger/0.6.1/env_logger/ for more
//! details.
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

#[macro_use]
extern crate failure;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

use std::env;

mod cli;

/// Entry point of the hyperion binary
fn main() {
    // Initialize logging, default to info
    let log_var_name = "HYPERION_LOG";
    if env::var(log_var_name).is_err() {
        env::set_var(log_var_name, "hyperion=info");
    }

    pretty_env_logger::init_custom_env(log_var_name);

    // Run CLI interface
    match cli::run() {
        Ok(_) => {}
        Err(err) => error!("{}", err),
    }
}

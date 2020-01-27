use std::path::PathBuf;
use std::str::FromStr;

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

#[derive(StructOpt)]
#[structopt(about = "Hyperion protocol daemon")]
pub struct Opt {
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

pub async fn main(opt: Opt) -> Result<(), HyperionError> {
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

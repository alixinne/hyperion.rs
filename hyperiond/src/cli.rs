//! Command line interface (CLI) of the hyperion binary

use clap::App;
use failure::ResultExt;

use hyperion::servers;

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;

use futures::{Future, Stream};
use stream_cancel::Tripwire;
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

use regex::Regex;

use crate::gui;

/// Error raised when the CLI fails
#[derive(Debug, Fail)]
pub enum CliError {
    #[fail(display = "no valid command specified, see hyperion --help for usage details")]
    /// An invalid subcommand was specified
    InvalidCommand,
    #[fail(display = "server error: {}", 0)]
    ServerError(String),
}

fn read_config<P: AsRef<Path>>(path: P) -> std::io::Result<hyperion::config::Configuration> {
    // Open file and create reader
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_yaml::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?)
}

/// Entry point for the hyperion CLI
///
/// Parses arguments for the command line and dispatches to the corresponding subcommands.
/// See cli.yml for the definition of subcommands and arguments.
pub fn run() -> Result<(), failure::Error> {
    // Parse CLI args
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .setting(clap::AppSettings::DisableHelpSubcommand)
        .setting(clap::AppSettings::GlobalVersion)
        .setting(clap::AppSettings::InferSubcommands)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .version(crate_version!())
        .get_matches();

    debug!("{} {}", crate_name!(), crate_version!());

    let configuration = read_config(matches.value_of("config").expect("--config is required"))?;

    if let Some(server_matches) = matches.subcommand_matches("server") {
        // Tripwire to cancel the server listening
        let (trigger, tripwire) = Tripwire::new();

        // Vector of server futures
        let bind_address = server_matches
            .value_of("bind-addr")
            .unwrap()
            .parse()
            .map_err(|_| CliError::ServerError("could not parse bind address".into()))?;

        let json_address = SocketAddr::new(
            bind_address,
            value_t!(server_matches, "json-port", u16)
                .context("json-port must be a port number")?,
        );

        let proto_address = SocketAddr::new(
            bind_address,
            value_t!(server_matches, "proto-port", u16)
                .context("proto-port must be a port number")?,
        );

        let disable_devices = server_matches
            .value_of("disable-devices")
            .map(|value| {
                Regex::new(value).expect("failed to parse regex, please see https://docs.rs/regex/1.1.6/regex/#syntax for details")
            });

        let (debug_listener, _debug_window) = if server_matches.occurrences_of("gui") > 0 {
            gui::build_listener()
        } else {
            (None, None)
        };

        let (hyperion, sender) =
            hyperion::hyperion::Hyperion::new(configuration, disable_devices, debug_listener)?;

        let servers = vec![
            servers::bind_json(&json_address, sender.clone(), tripwire.clone())?,
            servers::bind_proto(&proto_address, sender.clone(), tripwire.clone())?,
        ];

        let server_future = futures::future::join_all(servers).map(|_| ());

        tokio::run(futures::lazy(move || {
            tokio::spawn(
                Signal::new(SIGINT)
                    .flatten_stream()
                    .map(|_| "SIGINT")
                    .select(Signal::new(SIGTERM).flatten_stream().map(|_| "SIGTERM"))
                    .into_future()
                    .and_then(move |(signal, _): (Option<&'static str>, _)| {
                        info!("got {}, terminating", signal.unwrap());
                        drop(trigger);
                        Ok(())
                    })
                    .map_err(|_| {
                        panic!("ctrl_c error");
                    }),
            );

            tokio::spawn(
                hyperion
                    .map_err(|e| {
                        warn!("hyperion error: {}", e);
                    })
                    .select(tripwire.clone())
                    .map(|_| ())
                    .map_err(|_| {
                        error!("hyperion tripwire error");
                    }),
            );

            server_future
                .map_err(|error| {
                    warn!("server error: {:?}", error);
                })
                .select(tripwire)
                .map(|_| {
                    info!("server terminating");
                })
                .map_err(|_| {
                    panic!("select failure");
                })
        }));

        Ok(())
    } else {
        bail!(CliError::InvalidCommand)
    }
}

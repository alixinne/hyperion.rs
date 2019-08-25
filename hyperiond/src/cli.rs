//! Command line interface (CLI) of the hyperion binary

use clap::App;
use failure::{Fail, ResultExt};

use hyperion::servers;

use std::net::SocketAddr;

use std::sync::{atomic::AtomicI32, atomic::Ordering, Arc};

use futures::{Future, Stream};
use stream_cancel::Tripwire;
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

/// Error raised when the CLI fails
#[derive(Debug, Fail)]
pub enum CliError {
    /// An invalid subcommand was specified
    #[fail(display = "no valid command specified, see hyperion --help for usage details")]
    InvalidCommand,
    /// Hyperion server error
    #[fail(display = "server error: {}", 0)]
    ServerError(String),
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

    let config_path = matches
        .value_of("config")
        .expect("--config is required")
        .to_owned();

    let config = hyperion::config::Config::read(&config_path).map_err(|error| {
        let context = format!("{}: {}", config_path, error);
        error.context(context)
    })?;

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

        let web_address = SocketAddr::new(
            bind_address,
            value_t!(server_matches, "web-port", u16).context("web-port must be a port number")?,
        );

        let config = config.into_handle();

        let (hyperion, sender) = hyperion::hyperion::Service::new(config.clone(), None)?;

        let servers = vec![
            servers::bind_json(&json_address, sender.clone(), tripwire.clone())?,
            servers::bind_proto(&proto_address, sender.clone(), tripwire.clone())?,
        ];

        let server_future = futures::future::join_all(servers).map(|_| ());

        let (sender, receiver) = futures::sync::oneshot::channel::<()>();

        // Instantiate the web server
        let web_server = hyperion::web::bind(web_address, receiver, "web/dist", config);

        let exit_code = Arc::new(AtomicI32::new(exitcode::OK));
        let final_exit_code = exit_code.clone();

        tokio::run(futures::lazy(move || {
            tokio::spawn(
                Signal::new(SIGINT)
                    .flatten_stream()
                    .select(Signal::new(SIGTERM).flatten_stream())
                    .into_future()
                    .and_then(move |(signal, _): (Option<i32>, _)| {
                        let signal_name = if let Some(signal_number) = signal {
                            if signal_number == SIGINT {
                                exit_code.store(130, Ordering::SeqCst);
                                "SIGINT".to_owned()
                            } else if signal_number == SIGTERM {
                                "SIGTERM".to_owned()
                            } else {
                                signal_number.to_string()
                            }
                        } else {
                            "<unknown>".to_owned()
                        };

                        info!("got {}, terminating", signal_name);
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

            tokio::spawn(web_server);

            server_future
                .map_err(|error| {
                    warn!("server error: {:?}", error);
                })
                .select(tripwire)
                .map(move |_| {
                    sender.send(()).expect("failed to signal the web server");
                    info!("server terminating");
                })
                .map_err(|_| {
                    panic!("select failure");
                })
        }));

        std::process::exit(final_exit_code.load(Ordering::SeqCst));
    } else {
        bail!(CliError::InvalidCommand)
    }
}

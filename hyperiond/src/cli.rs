//! Command line interface (CLI) of the hyperion binary

use clap::App;
use error_chain::bail;

use hyperion::servers;

use std::net::SocketAddr;

use std::sync::{atomic::AtomicI32, atomic::Ordering, Arc};

use futures::{Future, Stream};
use stream_cancel::Tripwire;
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

mod errors {
    use error_chain::error_chain;

    error_chain! {
        links {
            ConfigLoad(::hyperion::config::ConfigLoadError, ::hyperion::config::ConfigLoadErrorKind);
            JsonServer(::hyperion::servers::json::JsonServerError, ::hyperion::servers::json::JsonServerErrorKind);
            ProtoServer(::hyperion::servers::proto::ProtoServerError, ::hyperion::servers::proto::ProtoServerErrorKind);
            Host(::hyperion::runtime::host::Error, ::hyperion::runtime::host::ErrorKind);
        }

        errors {
            InvalidCommand {
                description("invalid command")
                display("no valid command specified, see hyperion --help for usage details")
            }

            ServerError(m: String) {
                description("server launch error")
                display("server error: {}", m)
            }
        }
    }
}

use errors::*;

/// Entry point for the hyperion CLI
///
/// Parses arguments for the command line and dispatches to the corresponding subcommands.
/// See cli.yml for the definition of subcommands and arguments.
pub fn run() -> Result<()> {
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

    let config = hyperion::config::Config::read(&config_path).chain_err(|| config_path)?;

    if let Some(server_matches) = matches.subcommand_matches("server") {
        // Tripwire to cancel the server listening
        let (trigger, tripwire) = Tripwire::new();

        // Vector of server futures
        let bind_address = server_matches
            .value_of("bind-addr")
            .unwrap()
            .parse()
            .chain_err(|| "could not parse bind address")?;

        let json_address = SocketAddr::new(
            bind_address,
            value_t!(server_matches, "json-port", u16)
                .chain_err(|| "json-port must be a port number")?,
        );

        let proto_address = SocketAddr::new(
            bind_address,
            value_t!(server_matches, "proto-port", u16)
                .chain_err(|| "proto-port must be a port number")?,
        );

        let web_address = SocketAddr::new(
            bind_address,
            value_t!(server_matches, "web-port", u16)
                .chain_err(|| "web-port must be a port number")?,
        );

        let config = config.into_handle();
        let (sender, receiver) = futures::sync::mpsc::unbounded();
        let host = hyperion::runtime::Host::new(receiver, sender, config)?;

        let hyperion = hyperion::hyperion::Service::new(host.clone(), None);

        let servers = vec![
            servers::bind_json(&json_address, host.clone(), tripwire.clone())?,
            servers::bind_proto(&proto_address, host.clone(), tripwire.clone())?,
        ];

        let server_future = futures::future::join_all(servers).map(|_| ());

        let (sender, receiver) = futures::sync::oneshot::channel::<()>();

        // Instantiate the web server
        let web_server = hyperion::web::bind(web_address, receiver, "web/dist", host.clone());

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
        bail!(ErrorKind::InvalidCommand)
    }
}

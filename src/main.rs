#[macro_use]
extern crate log;

use structopt::StructOpt;
use tokio::runtime::Runtime;
use tokio::signal;

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u32,
}

#[paw::main]
fn main(opts: Opts) -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    env_logger::Builder::from_env(
        env_logger::Env::new()
            .filter_or(
                "HYPERION_LOG",
                match opts.verbose {
                    0 => "hyperion=warn",
                    1 => "hyperion=info",
                    2 => "hyperion=debug",
                    _ => "hyperion=trace",
                },
            )
            .write_style("HYPERION_LOG_STYLE"),
    )
    .try_init()
    .ok();

    // Connect to database
    let db = hyperion::db::Db::try_default()?;

    // Load configuration
    let config = hyperion::models::Config::load(&db)?;

    // Create tokio runtime
    let rt = Runtime::new()?;

    rt.block_on(async move {
        // Create the global state object
        let global = hyperion::global::GlobalData::new().wrap();

        // Start the flatbuffers servers
        if config.global.flatbuffers_server.enable {
            tokio::spawn({
                let global = global.clone();
                let config = config.global.flatbuffers_server.clone();

                async move {
                    let result = hyperion::servers::bind(
                        config,
                        global,
                        hyperion::servers::flat::handle_client,
                    )
                    .await;

                    if let Err(error) = result {
                        error!("Flatbuffers server terminated: {:?}", error);
                    }
                }
            });
        }

        // Start the JSON server
        tokio::spawn({
            let global = global.clone();
            let config = config.global.json_server.clone();

            async move {
                let result =
                    hyperion::servers::bind(config, global, hyperion::servers::json::handle_client)
                        .await;

                if let Err(error) = result {
                    error!("JSON server terminated: {:?}", error);
                }
            }
        });

        // Start the Protobuf server
        if config.global.proto_server.enable {
            tokio::spawn({
                let global = global.clone();
                let config = config.global.proto_server.clone();

                async move {
                    let result = hyperion::servers::bind(
                        config,
                        global,
                        hyperion::servers::proto::handle_client,
                    )
                    .await;

                    if let Err(error) = result {
                        error!("Protobuf server terminated: {:?}", error);
                    }
                }
            });
        }

        // Receive global updates
        // Create this reader first so the muxer does not complain it can't send its output
        let mut reader = global.subscribe_muxed().await;

        // Spawn priority muxer
        let muxer = hyperion::muxer::PriorityMuxer::new(global.clone()).await;
        tokio::spawn(muxer.run());

        // Should we continue running?
        let mut abort = false;

        while !abort {
            tokio::select! {
                update = reader.recv() => {
                    info!("update: {:?}", update);
                }
                _ = signal::ctrl_c() => {
                    abort = true;
                }
            }
        }

        Ok(())
    })
}

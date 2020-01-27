use env_logger::Env;
use structopt::StructOpt;
use tokio::runtime::Builder;

mod cli;

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"), author, about)]
enum Opt {
    Client(cli::client::Opt),
    Server(cli::server::Opt),
}

#[paw::main]
fn main(opt: Opt) -> Result<(), cli::CliError> {
    // Initialize logging, default to info
    env_logger::from_env(Env::default().filter_or("HYPERION_LOG", "hyperion=info")).init();

    match opt {
        Opt::Client(client_opts) => {
            Ok(cli::client::main(client_opts)?)
        }
        Opt::Server(server_opts) => {
            let mut runtime = Builder::new()
                .threaded_scheduler()
                .core_threads(2)
                .thread_name("hyperion_core")
                .build()
                .expect("failed to build the tokio runtime");

            Ok(runtime.block_on(cli::server::main(server_opts))?)
        }
    }
}

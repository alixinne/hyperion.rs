use env_logger::Env;
use structopt::StructOpt;

mod cli;

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"), author, about)]
enum Opt {
    Client(cli::client::Opt),
    Server(cli::server::Opt),
}

#[paw::main]
#[tokio::main(core_threads = 2, max_threads = 2)]
async fn main(opt: Opt) -> Result<(), cli::CliError> {
    // Initialize logging, default to info
    env_logger::from_env(Env::default().filter_or("HYPERION_LOG", "hyperion=info")).init();

    match opt {
        Opt::Client(client_opts) => Ok(cli::client::main(client_opts)?),
        Opt::Server(server_opts) => Ok(cli::server::main(server_opts).await?),
    }
}

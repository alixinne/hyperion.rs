use clap::App;
use failure::ResultExt;

use hyperion::server;

#[derive(Debug, Fail)]
pub enum CliError {
    #[fail(display = "no valid command specified, see hyperion --help for usage details")]
    InvalidCommand,
    #[fail(display = "server error: {}", 0)]
    ServerError(#[fail(cause)] server::ServerError),
}

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

    if let Some(server_matches) = matches.subcommand_matches("server") {
        server::server()
            .address(server_matches.value_of("bind-addr").unwrap().into())
            .json_port(
                value_t!(server_matches, "json-port", u16)
                    .context("json-port must be a port number")?,
            )
            .proto_port(
                value_t!(server_matches, "proto-port", u16)
                    .context("proto-port must be a port number")?,
            )
            .run()
            .map_err(|e| CliError::ServerError(e).into())
    } else {
        bail!(CliError::InvalidCommand)
    }
}

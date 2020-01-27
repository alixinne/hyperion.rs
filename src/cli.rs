pub mod client;
pub mod server;

use error_chain::error_chain;

error_chain! {
    types {
        CliError, CliErrorKind, ResultExt;
    }

    links {
        Server(server::HyperionError, server::HyperionErrorKind);
    }

    foreign_links {
        Io(std::io::Error);
    }
}

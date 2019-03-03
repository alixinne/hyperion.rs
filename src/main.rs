#[macro_use]
extern crate failure;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

use std::env;

mod cli;

fn main() {
    // Initialize logging, default to info
    let log_var_name = "HYPERION_LOG";
    if !env::var(log_var_name).is_ok() {
        env::set_var(log_var_name, "hyperion=info");
    }

    pretty_env_logger::init_custom_env(log_var_name);

    // Run CLI interface
    match cli::run() {
        Ok(_) => {},
        Err(err) => error!("{}", err)
    }
}

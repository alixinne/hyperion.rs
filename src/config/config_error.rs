//! ConfigError type definition
#![allow(missing_docs)]

use error_chain::error_chain;

error_chain! {
    types {
        ConfigError, ConfigErrorKind, ResultExt;
    }

    foreign_links {
        Io(std::io::Error);
        Save(atomicwrites::Error<serde_yaml::Error>);
        InvalidSyntax(serde_yaml::Error);
        InvalidConfig(validator::ValidationErrors);
    }
}

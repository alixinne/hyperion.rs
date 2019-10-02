//! ConfigLoadError type definition
#![allow(missing_docs)]

use error_chain::error_chain;

error_chain! {
    types {
        ConfigLoadError, ConfigLoadErrorKind, ResultExt;
    }

    foreign_links {
        Io(std::io::Error);
        InvalidSyntax(serde_yaml::Error);
        InvalidConfig(validator::ValidationErrors);
    }
}

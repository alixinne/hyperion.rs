//! Definition of the EffectError type
#![allow(missing_docs)]

use error_chain::error_chain;

error_chain! {
    types {
        EffectError, EffectErrorKind, ResultExt;
    }

    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        NotFound(name: String) {
            description("effect not found")
            display("effect '{}' was not found", name)
        }
    }
}

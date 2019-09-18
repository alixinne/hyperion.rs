//! Definition of the MethodError type
#![allow(missing_docs)]

use error_chain::error_chain;

error_chain! {
    types {
        MethodError, MethodErrorKind, ResultExt;
    }

    foreign_links {
        Io(::std::io::Error);
    }
}

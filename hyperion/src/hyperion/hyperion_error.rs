//! Definition of the HyperionError type
#![allow(missing_docs)]

use error_chain::error_chain;

use crate::methods;

error_chain! {
    types {
        HyperionError, HyperionErrorKind, ResultExt;
    }

    links {
        LedDeviceInit(methods::MethodError, methods::MethodErrorKind);
    }

    foreign_links {
        Tokio(tokio::timer::Error);
    }

    errors {
        ChannelReceive {
            description("failed to receive update from channel")
        }

        UpdaterPoll {
            description("failed to poll the updater interval")
        }
    }
}

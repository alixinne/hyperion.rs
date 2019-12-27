//! ServiceError type definition
#![allow(missing_docs)]

use error_chain::error_chain;

use crate::servers::{json, proto};

error_chain! {
    types {
        ServiceError, ServiceErrorKind, ResultExt;
    }

    links {
        JsonServer(json::JsonServerError, json::JsonServerErrorKind);
        ProtoServer(proto::ProtoServerError, proto::ProtoServerErrorKind);
    }
}

//! Server module
//!
//! Re-exports the definitions for the protobuf and JSON protocol server implementations of the
//! Hyperion software.

pub mod json;
pub use json::bind as bind_json;

pub mod proto;
pub use proto::bind as bind_proto;

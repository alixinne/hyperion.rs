//! Device communication methods definitions

mod from_endpoint;
pub use from_endpoint::*;

mod method;
pub use method::*;

mod method_error;
pub use method_error::*;

mod udp;
pub use udp::Udp;

mod stdout;
pub use stdout::Stdout;

mod ws;
pub use ws::Ws;

//! Definition of communcation methods

use crate::config::Endpoint;

mod method;
pub use method::*;

mod udp;
pub use udp::Udp;

mod stdout;
pub use stdout::Stdout;

mod ws;
pub use ws::Ws;

/// Build a method object from an endpoint specification
///
/// # Parameters
///
/// * `endpoint`: endpoint configuration
///
/// # Returns
///
/// Boxed `Method` trait object for the endpoint.
pub fn from_endpoint(endpoint: &Endpoint) -> Box<dyn Method + Send> {
    match endpoint {
        Endpoint::Stdout { bits } => Box::new(Stdout::new(*bits, "LED".to_owned())),
        Endpoint::Udp { address } => Box::new(Udp::new(address.clone())),
        Endpoint::Ws { address } => Box::new(Ws::new(address.clone())),
    }
}

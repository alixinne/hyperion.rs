//! Definition of the from_method function

use crate::config::Endpoint;

use super::*;

/// Box a method result
///
/// # Parameters
///
/// * `t`: result of the method initialization
fn to_box<T, E>(t: Result<T, E>) -> Result<Box<dyn Method + Send>, MethodError>
where
    T: Method + Send + 'static,
    MethodError: From<E>,
{
    t.map(|value| {
        let b: Box<dyn Method + Send> = Box::new(value);
        b
    })
    .map_err(MethodError::from)
}

/// Create a device method from its endpoint configuration
///
/// # Parameters
///
/// * `endpoint`: endpoint configuration to use
pub fn from_endpoint(endpoint: &Endpoint) -> Result<Box<dyn Method + Send>, MethodError> {
    trace!("creating method for {:?}", endpoint);

    match endpoint {
        Endpoint::Stdout { bits } => to_box(Stdout::new(*bits)),
        Endpoint::Udp { address } => to_box(Udp::new(address.to_owned())),
    }
}

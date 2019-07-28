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

use std::path::{Path, PathBuf};

/// Get the path to a script method by name
///
/// # Parameters
///
/// * `name`: name of the script method to find
fn method_path(name: &str) -> PathBuf {
    Path::new("scripts")
        .join("methods")
        .join(name.to_owned() + ".lua")
}

use serde_yaml::Value;
use std::collections::BTreeMap as Map;

/// Turn stdout parameters into stdout script parameters
///
/// # Parameters
///
/// * `bits`: output bit depth
fn stdout_params(bits: i32) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("bits".to_owned(), Value::Number(bits.into()));
    map
}

/// Create a device method from its endpoint configuration
///
/// # Parameters
///
/// * `endpoint`: endpoint configuration to use
pub fn from_endpoint(endpoint: &Endpoint) -> Result<Box<dyn Method + Send>, MethodError> {
    trace!("creating method for {:?}", endpoint);

    match endpoint {
        Endpoint::Stdout { bits } => {
            to_box(Script::new(&method_path("stdout"), stdout_params(*bits)))
        }
        Endpoint::Udp { address } => to_box(Udp::new(address.to_owned())),
        Endpoint::Script { path, params } => to_box(Script::new(path, params.to_owned())),
    }
}

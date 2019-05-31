//! Device communication methods definitions

use std::time::Instant;

use crate::config::Endpoint;
use crate::filters::ColorFilter;
use crate::runtime::{IdleTracker, LedInstance};

/// A method for communicating with a device
pub trait Method {
    /// Write the current LED status to the target device
    ///
    /// # Parameters
    ///
    /// * `time`: instant at which the filtered LED values should be evaluated
    /// * `filter`: filter to interpolate LED values
    /// * `leds`: reference to the LED state
    /// * `idle_tracker`: idle state tracker
    fn write(
        &self,
        time: Instant,
        filter: &ColorFilter,
        leds: &mut [LedInstance],
        idle_tracker: &mut IdleTracker,
    );
}

mod udp;
pub use udp::Udp;

mod script;
pub use script::Script;

/// Device method error
#[derive(Debug, Fail)]
pub enum MethodError {
    /// Wrapped I/O error
    #[fail(display = "I/O error: {}", error)]
    IoError {
        /// I/O error which triggered the MethodError
        error: std::io::Error,
    },
    /// Wrapped scripting engine error
    #[fail(display = "script error: {}", error)]
    ScriptError {
        /// I/O error which triggered the MethodError
        error: script::ScriptError,
    },
}

impl From<std::io::Error> for MethodError {
    fn from(error: std::io::Error) -> MethodError {
        MethodError::IoError { error }
    }
}

impl From<script::ScriptError> for MethodError {
    fn from(error: script::ScriptError) -> MethodError {
        MethodError::ScriptError { error }
    }
}

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

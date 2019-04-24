use crate::hyperion::{LedInstance, Endpoint};

pub trait Method {
    fn write(&self, leds: &[LedInstance]);
}

mod udp;
pub use udp::Udp;

mod script;
pub use script::Script;

#[derive(Debug, Fail)]
pub enum MethodError {
    #[fail(display = "I/O error: {}", error)]
    IoError { error: std::io::Error },
    #[fail(display = "script error: {}", error)]
    ScriptError { error: script::ScriptError },
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

fn to_box<T, E>(t: Result<T, E>) -> Result<Box<dyn Method + Send>, MethodError>
    where T: Method + Send + 'static,
          MethodError: From<E>
{
    t.map(|value| {
        let b: Box<dyn Method + Send> = Box::new(value);
        b
    })
    .map_err(MethodError::from)
}

use std::path::{Path, PathBuf};
fn method_path(name: &str) -> PathBuf {
    Path::new("scripts").join("methods").join(name.to_owned() + ".lua")
}

use serde_json::Value;
use std::collections::BTreeMap as Map;
fn stdout_params(bits: i32) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("bits".to_owned(), Value::Number(bits.into()));
    map
}

pub fn from_endpoint(endpoint: &Endpoint) -> Result<Box<dyn Method + Send>, MethodError> {
    match endpoint {
        Endpoint::Stdout { bits } => to_box(Script::new(&method_path("stdout"), stdout_params(*bits))),
        Endpoint::Udp { address } => to_box(Udp::new(address.to_owned())),
        Endpoint::Script { path, params } => to_box(Script::new(path, params.to_owned())),
    }
}

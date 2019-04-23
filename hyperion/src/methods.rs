use crate::hyperion::{LedInstance, Endpoint};

pub trait Method {
    fn write(&self, leds: &[LedInstance]);
}

mod stdout;
pub use stdout::Stdout;

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

pub fn from_endpoint(endpoint: &Endpoint) -> Result<Box<dyn Method + Send>, MethodError> {
    match endpoint {
        Endpoint::Stdout => Ok(Box::new(Stdout::new())),
        Endpoint::Udp { address } => to_box(Udp::new(address.to_owned())),
        Endpoint::Script { path, params } => to_box(Script::new(path.to_owned(), params.to_owned())),
    }
}

use crate::hyperion::{Led, Endpoint};

pub trait Method {
    fn write(&self, leds: &[Led]);
}

mod stdout;
pub use stdout::Stdout;

mod udp;
pub use udp::Udp;

mod script;
pub use script::Script;

pub fn from_endpoint(endpoint: Endpoint) -> Box<dyn Method> {
    match endpoint {
        Endpoint::Stdout => Box::new(Stdout::new()),
        Endpoint::Udp { address } => Box::new(Udp::new(address)),
        Endpoint::Script { path, params } => Box::new(Script::new(path, params).unwrap()),
    }
}

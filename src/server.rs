use tokio;

use std::net::SocketAddr;

mod json;
mod proto;

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "failed to bind to the specified address, is it already in use?")]
    BindFailed,
    #[fail(display = "failed to parse address: {}", 0)]
    AddrParseFailed(#[fail(cause)] std::net::AddrParseError),
}

pub struct Builder {
    address: String,
    json_port: u16,
    proto_port: u16,
}

impl Builder {
    pub fn run(self) -> Result<(), ServerError> {
        let address = self
            .address
            .parse()
            .map_err(|e| ServerError::AddrParseFailed(e))?;

        let json_address = SocketAddr::new(address, self.json_port);
        let proto_address = SocketAddr::new(address, self.proto_port);

        let json_server = json::bind(&json_address).map_err(|_e| ServerError::BindFailed)?;
        let proto_server = proto::bind(&proto_address).map_err(|_e| ServerError::BindFailed)?;

        tokio::run(futures::lazy(|| {
            tokio::spawn(json_server);
            tokio::spawn(proto_server);

            Ok(())
        }));

        Ok(())
    }

    pub fn address(self, address: String) -> Self {
        Builder { address, ..self }
    }

    pub fn json_port(self, json_port: u16) -> Self {
        Builder { json_port, ..self }
    }

    pub fn proto_port(self, proto_port: u16) -> Self {
        Builder { proto_port, ..self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            address: String::from("127.0.0.1"),
            json_port: 19444,
            proto_port: 19445,
        }
    }
}

pub fn server() -> Builder {
    Builder::default()
}

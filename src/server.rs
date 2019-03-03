//! Server module
//!
//! Contains the definitions for the protobuf and JSON protocol server implementations of the
//! Hyperion software, as well as an abstract wrapper for these.

use tokio;

use std::net::SocketAddr;

mod json;
mod proto;

/// Error raised when the server fails
#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "failed to bind to the specified address, is it already in use?")]
    BindFailed,
    #[fail(display = "failed to parse address: {}", 0)]
    AddrParseFailed(#[fail(cause)] std::net::AddrParseError),
}

/// Builder object for the server wrapper
pub struct Builder {
    address: String,
    json_port: u16,
    proto_port: u16,
}

impl Builder {
    /// Runs the server wrapper using the provided parameters
    ///
    /// This method will block forever, as it enters the tokio runtime.
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

    /// Set the listening address for the server
    ///
    /// Defaults to 127.0.0.1
    ///
    /// # Parameters
    ///
    /// * `address`: IPv4 (or IPv6) address to listen on
    pub fn address(self, address: String) -> Self {
        Builder { address, ..self }
    }

    /// Set the listening port for the JSON protocol server
    ///
    /// Defaults to 19444
    ///
    /// # Parameters
    ///
    /// * `json_port`: port number for the JSON protocol server
    pub fn json_port(self, json_port: u16) -> Self {
        Builder { json_port, ..self }
    }

    /// Set the listening port for the protobuf protocol server
    ///
    /// Defaults to 19445
    ///
    /// # Parameters
    ///
    /// * `proto_port`: port number for the protobuf protocol server
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

/// Start building a new server wrapper using its `Builder`
pub fn server() -> Builder {
    Builder::default()
}

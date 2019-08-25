//! Web server bind method definition

use std::net::SocketAddr;
use std::path::PathBuf;

use futures::Future;
use hyper::Server;

use crate::config::ConfigurationHandle;
use super::routes::build_router;

/// Web server graceful shutdown signal
pub type GracefulShutdownReceiver = futures::sync::oneshot::Receiver<()>;

/// Server future type
pub type ServerFuture = Box<dyn Future<Item = (), Error = ()> + Send>;

/// Bind the web server to the given address and return the corresponding future
///
/// # Parameters
///
/// * `addr`: address to bind the server to
/// * `shutdown`: channel receiver to signal the web server should shutdown
pub fn bind<P: Into<PathBuf> + Send + 'static>(
    addr: SocketAddr,
    shutdown: GracefulShutdownReceiver,
    webroot: P,
    configuration: ConfigurationHandle,
) -> ServerFuture {
    Box::new(futures::lazy(move || {
        let server = Server::bind(&addr).serve(build_router(webroot.into(), configuration));

        let server = server.with_graceful_shutdown(shutdown).map_err(|err| {
            error!("web server error: {}", err);
        });

        info!("listening on http://{}", addr);
        server
    }))
}

//! LoggingServer type definition

use futures::Future;
use hyper::{
    http,
    service::{MakeService, Service},
};

use super::routes::Router;

/// A service that wraps the router and logs requests with their response
#[derive(Clone)]
pub struct LoggingServer {
    /// Internal router service
    router: Router,
}

impl LoggingServer {
    /// Create a new LoggingServer
    ///
    /// # Parameters
    ///
    /// * `router`: router instance to perform actual requests
    pub fn new(router: Router) -> Self {
        Self { router }
    }
}

impl Service for LoggingServer {
    type Error = reset_router::Never;
    type Future = Box<Future<Item = http::Response<Self::ResBody>, Error = Self::Error> + Send>;
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;

    fn call(&mut self, req: http::Request<Self::ReqBody>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().clone();
        let version = req.version();

        Box::new(self.router.call(req).then(move |result| {
            let status = if let Ok(response) = &result {
                response.status()
            } else {
                http::StatusCode::INTERNAL_SERVER_ERROR
            };

            let content_type = if let Ok(Some(header_value)) = result
                .as_ref()
                .map(|response| response.headers().get(http::header::CONTENT_TYPE))
            {
                header_value.as_bytes()
            } else {
                b""
            };

            debug!(
                "{method} {path} {version:?} {status} {content_type}",
                method = method,
                path = path,
                version = version,
                status = status,
                content_type = String::from_utf8_lossy(content_type)
            );

            result
        }))
    }
}

impl<Ctx> MakeService<Ctx> for LoggingServer {
    type Error = <LoggingServer as Service>::Error;
    type Future = futures::future::FutureResult<LoggingServer, reset_router::Never>;
    type MakeError = reset_router::Never;
    type ReqBody = <LoggingServer as Service>::ReqBody;
    type ResBody = <LoggingServer as Service>::ResBody;
    type Service = LoggingServer;

    fn make_service(&mut self, _: Ctx) -> Self::Future {
        futures::future::ok(self.clone())
    }
}

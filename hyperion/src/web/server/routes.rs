//! Web server routes

use std::path::PathBuf;

use futures::Future;
use hyper::{header, http, Body, StatusCode};
use hyper_staticfile::Static;
use reset_router::{bits::Method, Request, RequestExtensions, Response, Router};

use crate::config::ConfigurationHandle;

/// Hyper server state
#[derive(Clone)]
pub struct State {
    /// Static file server
    hyper_static: Static,
    /// Reference to the configuration
    configuration: ConfigurationHandle,
}

/// Crate version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// /api/server route
fn api_server(_: Request) -> Result<Response, Response> {
    let response = http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "version": VERSION,
            "hostname": hostname::get_hostname(),
        }).to_string()))
        .unwrap();

    Ok(response)
}

impl State {
    /// Create a new Hyper server state
    ///
    /// # Parameters
    ///
    /// * `webroot`: path to the root for static files
    /// * `configuration`: configuration handle
    pub fn new(webroot: PathBuf, configuration: ConfigurationHandle) -> Self {
        Self {
            hyper_static: Static::new(webroot),
            configuration,
        }
    }
}

/// Build the hyperion.rs web router
///
/// # Parameters
///
/// * `webroot`: path to the root for static files
/// * `configuration`: configuration handle
pub fn build_router(webroot: PathBuf, configuration: ConfigurationHandle) -> Router<State> {
    Router::build()
        .with_state(State::new(webroot, configuration))
        .add(Method::GET, r"^/api/server$", api_server)
        .add(Method::GET, r"", |req: Request| {
            req.state::<State>()
                .unwrap()
                .hyper_static
                .serve(req)
                .map_err(|_err| {
                    http::Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::default())
                        .unwrap()
                })
        })
        .finish()
        .unwrap()
}

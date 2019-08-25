//! Web server routes

use std::path::PathBuf;

use futures::Future;
use hyper::{header, http, Body, StatusCode};
use hyper_staticfile::Static;
use reset_router::{bits::Method, Request, RequestExtensions, Response, Router};

use crate::config::ConfigHandle;

/// Hyper server state
#[derive(Clone)]
pub struct State {
    /// Static file server
    hyper_static: Static,
    /// Reference to the configuration
    config: ConfigHandle,
}

/// Crate version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// /api/server route
fn api_server(_: Request) -> Result<Response, Response> {
    let response = http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "version": VERSION,
                "hostname": hostname::get_hostname(),
            })
            .to_string(),
        ))
        .unwrap();

    Ok(response)
}

/// /api/devices route
fn api_devices(req: Request) -> Result<Response, Response> {
    let response = http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&req.state::<State>().unwrap().config.read().unwrap().devices)
                .unwrap(),
        ))
        .unwrap();

    Ok(response)
}

impl State {
    /// Create a new Hyper server state
    ///
    /// # Parameters
    ///
    /// * `webroot`: path to the root for static files
    /// * `config`: configuration handle
    pub fn new(webroot: PathBuf, config: ConfigHandle) -> Self {
        Self {
            hyper_static: Static::new(webroot),
            config,
        }
    }
}

/// Build the hyperion.rs web router
///
/// # Parameters
///
/// * `webroot`: path to the root for static files
/// * `config`: configuration handle
pub fn build_router(webroot: PathBuf, config: ConfigHandle) -> Router<State> {
    Router::build()
        .with_state(State::new(webroot, config))
        .add(Method::GET, r"^/api/server$", api_server)
        .add(Method::GET, r"^/api/devices$", api_devices)
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

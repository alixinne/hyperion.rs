//! Web server routes

use std::path::PathBuf;

use futures::{Future, Stream};
use hyper::{header, http, Body, StatusCode};
use hyper_staticfile::Static;
use reset_router::{bits::Method, Request, RequestExtensions, Response};

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

macro_rules! json_response {
    ($status_code:expr, $body:expr) => {
        // TODO: Handle unwrap?
        http::Response::builder()
            .status($status_code)
            .body(Body::from(serde_json::to_string($body).unwrap()))
            .unwrap()
    };
}

macro_rules! json_error {
    ($error:expr) => { json_response!(StatusCode::BAD_REQUEST, &json!({ "error": $error.to_string() })) }
}

macro_rules! json_try {
    ($what:expr) => {
        $what.map_err(|error| json_error!(error))?
    };
}

/// GET /api/server route
fn api_server(_: Request) -> Result<Response, Response> {
    Ok(json_response!(
        StatusCode::OK,
        &json!({ "version": VERSION, "hostname": hostname::get_hostname() })
    ))
}

/// GET /api/devices route
fn api_devices(req: Request) -> Result<Response, Response> {
    Ok(json_response!(
        StatusCode::OK,
        &req.state::<State>().unwrap().config.read().unwrap().devices
    ))
}

/// PATCH /api/devices/:id route
fn api_patch_device(req: Request) -> Box<impl Future<Item = Response, Error = Response>> {
    let (id,) = req.parsed_captures::<(usize,)>().unwrap();
    let (parts, body) = req.into_parts();

    Box::new(
        body.concat2()
            .map_err(|error| json_error!(error))
            .and_then(move |body| {
                let parsed: crate::config::DeviceUpdate = json_try!(serde_json::from_slice(&body));

                let state = parts.state::<State>().unwrap();
                let device = &mut state.config.write().unwrap().devices[id];
                json_try!(device.update(parsed));

                Ok(json_response!(StatusCode::OK, device))
            }),
    )
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

/// hyperion.rs router type
pub type Router = reset_router::Router<State>;

/// Build the hyperion.rs web router
///
/// # Parameters
///
/// * `webroot`: path to the root for static files
/// * `config`: configuration handle
pub fn build_router(webroot: PathBuf, config: ConfigHandle) -> Router {
    reset_router::Router::build()
        .with_state(State::new(webroot, config))
        .add(Method::GET, r"^/api/server$", api_server)
        .add(Method::GET, r"^/api/devices$", api_devices)
        .add(Method::PATCH, r"^/api/devices/(\d+)$", api_patch_device)
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

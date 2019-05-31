//! `hyperion` is the Rust crate implementing the core features of the
//! [Hyperion](https://github.com/hyperion-project/hyperion) ambient lighting software.
//!
//! # Structure
//!
//! The different components of this crate are implemented as futures which are run using a tokio
//! runtime by the `hyperiond` program. The various components are:
//!
//! * Servers: respond to requests from Hyperion clients (either JSON or protobuf)
//! * Hyperion instance: handles state updates from servers and effects, and forwards them to
//! devices
//!
//! These components are backed by methods, which implement the actual protocol used to talk to LED
//! devices. Methods can be written in Rust and compiled in to this crate, or as extensible Lua
//! scripts using the provided API (work in progress).
//!
//! # Authors
//!
//! * [Vincent Tavernier](https://github.com/vtavernier)
//!
//! # License
//!
//! This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)

#[macro_use]
extern crate failure;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod filters;
pub mod hyperion;
pub mod image;
pub mod methods;
pub mod runtime;
pub mod servers;

//! `hyperion` is the Rust crate implementing the core features of the
//! [Hyperion](https://github.com/hyperion-project/hyperion) ambient lighting software.
//!
//! # Structure
//!
//! This is a complete rewrite of the previous hyperion.rs code using `async/await` and
//! tokio 0.2. This documentation will be updated when the program structure is stabilized.
//!
//! # Authors
//!
//! * [Vincent Tavernier](https://github.com/vtavernier)
//!
//! # License
//!
//! This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![recursion_limit = "512"]

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate validator_derive;

pub mod color;
pub mod config;
pub mod filters;
pub mod hyperion;
pub mod image;
pub mod methods;
pub mod runtime;
pub mod serde;
pub mod servers;

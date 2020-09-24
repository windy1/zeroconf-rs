//! macOS-specific ZeroConf bindings
//!
//! This module wraps the [Bonjour] mDNS implementation which is distributed with macOS.
//!
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)

pub(crate) mod browser;
pub(crate) mod service;

pub mod compat;
pub mod service_ref;

pub use browser::*;
pub use service::*;

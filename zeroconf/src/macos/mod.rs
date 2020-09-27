//! macOS-specific ZeroConf bindings
//!
//! This module wraps the [Bonjour] mDNS implementation which is distributed with macOS.
//!
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)

pub(crate) mod browser;
pub(crate) mod constants;
pub(crate) mod event_loop;
pub(crate) mod service;
pub(crate) mod txt_record;

pub mod bonjour_util;
pub mod service_ref;
pub mod txt_record_ref;

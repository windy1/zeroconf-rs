//! macOS-specific ZeroConf bindings
//!
//! This module wraps the [Bonjour] mDNS implementation which is distributed with macOS.
//!
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)

pub(crate) mod constants;

pub mod bonjour_util;
pub mod browser;
pub mod event_loop;
pub mod service;
pub mod service_ref;
pub mod txt_record;
pub mod txt_record_ref;

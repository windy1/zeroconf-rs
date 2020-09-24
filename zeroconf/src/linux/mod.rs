//! Linux-specific ZeroConf bindings
//!
//! This module wraps the [Avahi] mDNS implementation which can be found in most major Linux
//! distributions. It is a sufficient (and often more featured) replacement for Apple's [Bonjour].
//!
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
//! [Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)

pub(crate) mod browser;
pub(crate) mod constants;
pub(crate) mod service;

pub mod avahi_util;
pub mod client;
pub mod entry_group;
pub mod poll;
pub mod raw_browser;
pub mod resolver;

pub use browser::*;
pub use service::*;

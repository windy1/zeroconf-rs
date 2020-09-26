//! `zeroconf` is a cross-platform library that wraps underlying [ZeroConf/mDNS] implementations
//! such as [Bonjour] or [Avahi], providing an easy and idiomatic way to both register and
//! browse services.
//!
//! This crate provides the cross-platform [`MdnsService`] and [`MdnsBrowser`] available for each
//! supported platform as well as platform-specific modules for lower-level access to the mDNS
//! implementation should that be necessary.
//!
//! Most users of this crate need only [`MdnsService`] and [`MdnsBrowser`].
//!
//! # Examples
//!
//! ## Register a service
//!
//! When registering a service, you may optionally pass a "context" to pass state through the
//! callback. The only requirement is that this context implements the [`Any`] trait, which most
//! types will automatically. See [`MdnsService`] for more information about contexts.
//!
//! ```no_run
//! use std::any::Any;
//! use std::sync::{Arc, Mutex};
//! use zeroconf::{MdnsService, ServiceRegistration};
//!
//! #[derive(Default, Debug)]
//! pub struct Context {
//!     service_name: String,
//! }
//!
//! fn main() {
//!     let mut service = MdnsService::new("_http._tcp", 8080);
//!     let context: Arc<Mutex<Context>> = Arc::default();
//!
//!     service.set_registered_callback(Box::new(on_service_registered));
//!     service.set_context(Box::new(context));
//!
//!     // blocks current thread, must keep-alive to keep service active
//!     service.start().unwrap();
//! }
//!
//! fn on_service_registered(
//!     result: zeroconf::Result<ServiceRegistration>,
//!     context: Option<Arc<dyn Any>>,
//! ) {
//!     let service = result.unwrap();
//!
//!     println!("Service registered: {:?}", service);
//!
//!     let context = context
//!         .as_ref()
//!         .unwrap()
//!         .downcast_ref::<Arc<Mutex<Context>>>()
//!         .unwrap()
//!         .clone();
//!
//!     context.lock().unwrap().service_name = service.name().clone();
//!
//!     println!("Context: {:?}", context);
//!
//!     // ...
//! }
//! ```
//!
//! ## Browsing services
//!
//! ```no_run
//! use std::any::Any;
//! use std::sync::Arc;
//! use zeroconf::{MdnsBrowser, ServiceDiscovery};
//!
//! fn main() {
//!     let mut browser = MdnsBrowser::new("_http._tcp");
//!
//!     browser.set_service_discovered_callback(Box::new(on_service_discovered));
//!
//!     // blocks current thread, must keep-alive to keep browser active
//!     browser.start().unwrap()
//! }
//!
//! fn on_service_discovered(
//!     result: zeroconf::Result<ServiceDiscovery>,
//!     _context: Option<Arc<dyn Any>>,
//! ) {
//!     println!("Service discovered: {:?}", result.unwrap());
//!
//!     // ...
//! }
//! ```
//!
//! [ZeroConf/mDNS]: https://en.wikipedia.org/wiki/Zero-configuration_networking
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
//! [Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)
//! [`MdnsService`]: struct.MdnsService.html
//! [`MdnsBrowser`]: struct.MdnsBrowser.html
//! [`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html

#![allow(clippy::needless_doctest_main)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate zeroconf_macros;
#[cfg(target_os = "linux")]
extern crate avahi_sys;
#[cfg(target_os = "macos")]
extern crate bonjour_sys;
#[macro_use]
extern crate derive_getters;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_new;
extern crate libc;

mod discovery;
mod registration;
#[macro_use]
mod macros;
mod interface;

pub mod builder;
pub mod error;
pub mod ffi;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

pub use discovery::*;
pub use error::Result;
pub use interface::*;
pub use registration::*;

/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_os = "linux")]
pub type MdnsBrowser = linux::browser::AvahiMdnsBrowser;
/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_os = "linux")]
pub type MdnsService = linux::service::AvahiMdnsService;
/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_os = "macos")]
pub type MdnsBrowser = macos::browser::BonjourMdnsBrowser;
/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_os = "macos")]
pub type MdnsService = macos::service::BonjourMdnsService;
/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(target_os = "macos")]
pub type EventLoop = macos::event_loop::BonjourEventLoop;

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
//! use std::time::Duration;
//! use zeroconf::{MdnsService, ServiceRegistration, TxtRecord};
//! use zeroconf::prelude::*;
//!
//! #[derive(Default, Debug)]
//! pub struct Context {
//!     service_name: String,
//! }
//!
//! fn main() {
//!     let mut service = MdnsService::new("_http._tcp", 8080);
//!     let mut txt_record = TxtRecord::new();
//!     let context: Arc<Mutex<Context>> = Arc::default();
//!
//!     txt_record.insert("foo", "bar").unwrap();
//!
//!     service.set_registered_callback(Box::new(on_service_registered));
//!     service.set_context(Box::new(context));
//!     service.set_txt_record(txt_record);
//!
//!     let event_loop = service.register().unwrap();
//!
//!     loop {
//!         // calling `poll()` will keep this service alive
//!         event_loop.poll(Duration::from_secs(0)).unwrap();
//!     }
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
//! ```no_run
//! use std::any::Any;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use zeroconf::{MdnsBrowser, ServiceDiscovery};
//! use zeroconf::prelude::*;
//!
//! fn main() {
//!     let mut browser = MdnsBrowser::new("_http._tcp");
//!
//!     browser.set_service_discovered_callback(Box::new(on_service_discovered));
//!
//!     let event_loop = browser.browse_services().unwrap();
//!
//!     loop {
//!         // calling `poll()` will keep this browser alive
//!         event_loop.poll(Duration::from_secs(0)).unwrap();
//!     }
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
//! [`MdnsService`]: type.MdnsService.html
//! [`MdnsBrowser`]: type.MdnsBrowser.html
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

#[macro_use]
#[cfg(test)]
#[allow(unused_imports)]
extern crate maplit;

#[macro_use]
mod macros;
mod interface;
#[cfg(test)]
mod tests;

pub mod browser;
pub mod error;
pub mod event_loop;
pub mod ffi;
pub mod prelude;
pub mod service;
pub mod txt_record;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

pub use browser::{ServiceDiscoveredCallback, ServiceDiscovery};
pub use interface::*;
pub use service::{ServiceRegisteredCallback, ServiceRegistration};

/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_os = "linux")]
pub type MdnsBrowser<Poll> = linux::browser::AvahiMdnsBrowser<Poll>;

/// Simple MdnsBrowser based on Avahi simple poll.
#[cfg(target_os = "linux")]
pub type SimpleMdnsBrowser = linux::browser::SimpleAvahiMdnsBrowser;

/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_os = "macos")]
pub type MdnsBrowser = macos::browser::BonjourMdnsBrowser;

/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_os = "linux")]
pub type MdnsService<Loop> = linux::service::AvahiMdnsService<Loop>;
#[cfg(target_os = "linux")]
pub type SimpleMdnsService = linux::service::SimpleAvahiMdnsService;
/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_os = "macos")]
pub type MdnsService = macos::service::BonjourMdnsService;

/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(target_os = "linux")]
pub type EventLoop = linux::event_loop::AvahiEventLoop;
/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(target_os = "macos")]
pub type EventLoop = macos::event_loop::BonjourEventLoop;

/// Type alias for the platform-specific structure responsible for storing and accessing TXT
/// record data
#[cfg(target_os = "linux")]
pub type TxtRecord = linux::txt_record::AvahiTxtRecord;
/// Type alias for the platform-specific structure responsible for storing and accessing TXT
/// record data
#[cfg(target_os = "macos")]
pub type TxtRecord = macos::txt_record::BonjourTxtRecord;

/// Result type for this library
pub type Result<T> = std::result::Result<T, error::Error>;

// TODO: Make a MacOS version of this too!
#[cfg(target_os = "linux")]
pub use linux::poll::TPoll;

#[cfg(target_os = "linux")]
pub use linux::poll::TNewPoll;

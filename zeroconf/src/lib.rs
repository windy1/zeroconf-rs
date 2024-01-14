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
//! #[macro_use]
//! extern crate log;
//!
//! use clap::Parser;
//!
//! use std::any::Any;
//! use std::sync::{Arc, Mutex};
//! use std::time::Duration;
//! use zeroconf::prelude::*;
//! use zeroconf::{MdnsService, ServiceRegistration, ServiceType, TxtRecord};
//!
//! #[derive(Parser, Debug)]
//! #[command(author, version, about)]
//! struct Args {
//!     /// Name of the service type to register
//!     #[clap(short, long, default_value = "http")]
//!     name: String,
//!
//!     /// Protocol of the service type to register
//!     #[clap(short, long, default_value = "tcp")]
//!     protocol: String,
//!
//!     /// Sub-types of the service type to register
//!     #[clap(short, long)]
//!     sub_types: Vec<String>,
//! }
//!
//! #[derive(Default, Debug)]
//! pub struct Context {
//!     service_name: String,
//! }
//!
//! fn main() {
//!     env_logger::init();
//!
//!     let Args {
//!         name,
//!         protocol,
//!         sub_types,
//!     } = Args::parse();
//!
//!     let sub_types = sub_types.iter().map(|s| s.as_str()).collect::<Vec<_>>();
//!     let service_type = ServiceType::with_sub_types(&name, &protocol, sub_types).unwrap();
//!     let mut service = MdnsService::new(service_type, 8080);
//!     let mut txt_record = TxtRecord::new();
//!     let context: Arc<Mutex<Context>> = Arc::default();
//!
//!     txt_record.insert("foo", "bar").unwrap();
//!
//!     service.set_name("zeroconf_example_service");
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
//!     info!("Service registered: {:?}", service);
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
//!     info!("Context: {:?}", context);
//!
//!     // ...
//! }
//! ```
//!
//! ## Browsing services
//! ```no_run
//! #[macro_use]
//! extern crate log;
//!
//! use clap::Parser;
//!
//! use std::any::Any;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use zeroconf::prelude::*;
//! use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};
//!
//! /// Example of a simple mDNS browser
//! #[derive(Parser, Debug)]
//! #[command(author, version, about)]
//! struct Args {
//!     /// Name of the service type to browse
//!     #[clap(short, long, default_value = "http")]
//!     name: String,
//!
//!     /// Protocol of the service type to browse
//!     #[clap(short, long, default_value = "tcp")]
//!     protocol: String,
//!
//!     /// Sub-type of the service type to browse
//!     #[clap(short, long)]
//!     sub_type: Option<String>,
//! }
//!
//! fn main() {
//!     env_logger::init();
//!
//!     let Args {
//!         name,
//!         protocol,
//!         sub_type,
//!     } = Args::parse();
//!
//!     let sub_types: Vec<&str> = match sub_type.as_ref() {
//!         Some(sub_type) => vec![sub_type],
//!         None => vec![],
//!     };
//!
//!     let service_type =
//!         ServiceType::with_sub_types(&name, &protocol, sub_types).expect("invalid service type");
//!
//!     let mut browser = MdnsBrowser::new(service_type);
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
//!     info!("Service discovered: {:?}", result.unwrap());
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
#[cfg(any(target_vendor = "apple", target_vendor = "pc"))]
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
mod ffi;
mod interface;
mod service_type;
#[cfg(test)]
mod tests;

pub mod browser;
pub mod error;
pub mod event_loop;
pub mod prelude;
pub mod service;
pub mod txt_record;

#[cfg(any(target_vendor = "apple", target_vendor = "pc"))]
pub mod bonjour;
#[cfg(target_os = "linux")]
pub mod linux;

pub use browser::{ServiceDiscoveredCallback, ServiceDiscovery};
pub use interface::*;
pub use service::{ServiceRegisteredCallback, ServiceRegistration};
pub use service_type::*;

/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_os = "linux")]
pub type MdnsBrowser = linux::browser::AvahiMdnsBrowser;
/// Type alias for the platform-specific mDNS browser implementation
#[cfg(any(target_vendor = "apple", target_vendor = "pc"))]
pub type MdnsBrowser = bonjour::browser::BonjourMdnsBrowser;

/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_os = "linux")]
pub type MdnsService = linux::service::AvahiMdnsService;
/// Type alias for the platform-specific mDNS service implementation
#[cfg(any(target_vendor = "apple", target_vendor = "pc"))]
pub type MdnsService = bonjour::service::BonjourMdnsService;

/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(target_os = "linux")]
pub type EventLoop<'a> = linux::event_loop::AvahiEventLoop<'a>;
/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(any(target_vendor = "apple", target_vendor = "pc"))]
pub type EventLoop<'a> = bonjour::event_loop::BonjourEventLoop<'a>;

/// Type alias for the platform-specific structure responsible for storing and accessing TXT
/// record data
#[cfg(target_os = "linux")]
pub type TxtRecord = linux::txt_record::AvahiTxtRecord;
/// Type alias for the platform-specific structure responsible for storing and accessing TXT
/// record data
#[cfg(any(target_vendor = "apple", target_vendor = "pc"))]
pub type TxtRecord = bonjour::txt_record::BonjourTxtRecord;

/// Result type for this library
pub type Result<T> = std::result::Result<T, error::Error>;

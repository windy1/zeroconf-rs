//! Trait definition for cross-platform browser

use crate::prelude::{TEventLoop, TTxtRecord};
use crate::{NetworkInterface, Result, ServiceType};
use std::any::Any;
use std::sync::Arc;

/// Event from [`MdnsBrowser`] received by the `ServiceBrowserCallback`.
///
/// [`MdnsBrowser`]: type.MdnsBrowser.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserEvent<TxtRecord> {
    Add(ServiceDiscovery<TxtRecord>),
    Remove(ServiceRemoval),
}

/// Interface for interacting with underlying mDNS implementation service browsing capabilities.
pub trait TMdnsBrowser {
    type EventLoop: TEventLoop;
    type TxtRecord: TTxtRecord;

    /// Creates a new `MdnsBrowser` that browses for the specified `kind` (e.g. `_http._tcp`)
    fn new(service_type: ServiceType) -> Self;

    /// Sets the network interface on which to browse for services on.
    ///
    /// Most applications will want to use the default value `NetworkInterface::Unspec` to browse
    /// on all available interfaces.
    fn set_network_interface(&mut self, interface: NetworkInterface);

    /// Returns the network interface on which to browse for services on.
    fn network_interface(&self) -> NetworkInterface;

    /// Sets the [`ServiceBrowserCallback`] that is invoked when the browser has discovered and
    /// resolved or removed a service.
    ///
    /// [`ServiceBrowserCallback`]: ../type.ServiceBrowserCallback.html
    fn set_service_callback(
        &mut self,
        service_callback: Box<ServiceBrowserCallback<Self::TxtRecord>>,
    );

    /// Sets the optional user context to pass through to the callback. This is useful if you need
    /// to share state between pre and post-callback. The context type must implement `Any`.
    fn set_context(&mut self, context: Box<dyn Any>);

    /// Returns the optional user context to pass through to the callback.
    fn context(&self) -> Option<&dyn Any>;

    /// Starts the browser. Returns an `EventLoop` which can be called to keep the browser alive.
    fn browse_services(&mut self) -> Result<Self::EventLoop>;
}

/// Callback invoked from [`MdnsBrowser`] once a service has been discovered and resolved or
/// removed.
///
/// # Arguments
/// * `browser_event` - The event received from Zeroconf
/// * `context` - The optional user context passed through
///
/// [`MdnsBrowser`]: type.MdnsBrowser.html
pub type ServiceBrowserCallback<TxtRecord> =
    dyn Fn(Result<BrowserEvent<TxtRecord>>, Option<Arc<dyn Any>>);

/// Represents a service that has been discovered by a [`MdnsBrowser`].
///
/// [`MdnsBrowser`]: type.MdnsBrowser.html
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// TODO: Restore derive(BuilderDelegate)
#[derive(Debug, Getters, Builder, Clone, PartialEq, Eq)]
pub struct ServiceDiscovery<TxtRecord> {
    name: String,
    service_type: ServiceType,
    domain: String,
    host_name: String,
    address: String,
    port: u16,
    txt: Option<TxtRecord>,
}

/// Represents a service that has been removed by a [`MdnsBrowser`].
///
/// [`MdnsBrowser`]: type.MdnsBrowser.html
#[derive(Debug, Getters, Builder, BuilderDelegate, Clone, PartialEq, Eq)]
pub struct ServiceRemoval {
    /// The "abc" part in "abc._http._udp.local"
    name: String,
    /// The "_http._udp" part in "abc._http._udp.local"
    kind: String,
    /// The "local" part in "abc._http._udp.local"
    domain: String,
}

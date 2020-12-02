//! Trait definition for cross-platform browser

use crate::{NetworkInterface, Result, TxtRecord, TNewPoll, TPoll};
use crate::event_loop::TEventLoop;
use std::any::Any;
use std::sync::Arc;
use std::fmt;

/// Interface for interacting with underlying mDNS implementation service browsing capabilities.
pub trait TMdnsBrowser<Poll> {
    /// Creates a new `MdnsBrowser` that browses for the specified `kind` (e.g. `_http._tcp`)
    fn new(kind: &str) -> Self;

    /// Sets the network interface on which to browse for services on.
    ///
    /// Most applications will want to use the default value `NetworkInterface::Unspec` to browse
    /// on all available interfaces.
    fn set_network_interface(&mut self, interface: NetworkInterface);

    /// Sets the [`ServiceDiscoveredCallback`] that is invoked when the browser has discovered and
    /// resolved a service.
    ///
    /// [`ServiceDiscoveredCallback`]: ../type.ServiceDiscoveredCallback.html
    fn set_service_discovered_callback(
        &mut self,
        service_discovered_callback: Box<ServiceDiscoveredCallback>,
    );

    /// Sets the optional user context to pass through to the callback. This is useful if you need
    /// to share state between pre and post-callback. The context type must implement `Any`.
    fn set_context(&mut self, context: Box<dyn Any>);

    /// Starts the browser. Returns an `EventLoop` which can be called to keep the browser alive.
    fn browse_services(&mut self) -> Result<Poll>
        where Poll: TNewPoll + TEventLoop + Clone + fmt::Debug;

    fn browse_services_with_poll(&mut self, poll: Poll) -> Result<()>
        where Poll: TPoll + fmt::Debug;
}

/// Callback invoked from [`MdnsBrowser`] once a service has been discovered and resolved.
///
/// # Arguments
/// * `discovered_service` - The service that was disovered
/// * `context` - The optional user context passed through
///
/// [`MdnsBrowser`]: type.MdnsBrowser.html
pub type ServiceDiscoveredCallback = dyn Fn(Result<ServiceDiscovery>, Option<Arc<dyn Any>>);

/// Represents a service that has been discovered by a [`MdnsBrowser`].
///
/// [`MdnsBrowser`]: type.MdnsBrowser.html
#[derive(Debug, Getters, Builder, BuilderDelegate, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ServiceDiscovery {
    name: String,
    kind: String,
    domain: String,
    host_name: String,
    address: String,
    port: u16,
    txt: Option<TxtRecord>,
}

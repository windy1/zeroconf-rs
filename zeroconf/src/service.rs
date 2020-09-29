//! Trait definition for cross-platform service.

use crate::{EventLoop, NetworkInterface, Result, ServiceRegisteredCallback, TxtRecord};
use std::any::Any;

/// Interface for interacting with underlying mDNS service implementation registration
/// capabilities.
pub trait TMdnsService {
    /// Creates a new `MdnsService` with the specified `kind` (e.g. `_http._tcp`) and `port`.
    fn new(kind: &str, port: u16) -> Self;

    /// Sets the name to register this service under.
    fn set_name(&mut self, name: &str);

    /// Sets the network interface to bind this service to.
    ///
    /// Most applications will want to use the default value `NetworkInterface::Unspec` to bind to
    /// all available interfaces.
    fn set_network_interface(&mut self, interface: NetworkInterface);

    /// Sets the domain on which to advertise the service.
    ///
    /// Most applications will want to use the default value of `ptr::null()` to register to the
    /// default domain.
    fn set_domain(&mut self, _domain: &str);

    /// Sets the SRV target host name.
    ///
    /// Most applications will want to use the default value of `ptr::null()` to use the machine's
    /// default host name.
    fn set_host(&mut self, _host: &str);

    /// Sets the optional `TxtRecord` to register this service with.
    fn set_txt_record(&mut self, txt_record: TxtRecord);

    /// Sets the [`ServiceRegisteredCallback`] that is invoked when the service has been
    /// registered.
    ///
    /// [`ServiceRegisteredCallback`]: ../type.ServiceRegisteredCallback.html
    fn set_registered_callback(&mut self, registered_callback: Box<ServiceRegisteredCallback>);

    /// Sets the optional user context to pass through to the callback. This is useful if you need
    /// to share state between pre and post-callback. The context type must implement `Any`.
    fn set_context(&mut self, context: Box<dyn Any>);

    /// Registers and start's the service. Returns an `EventLoop` which can be called to keep
    /// the service alive.
    fn register(&mut self) -> Result<EventLoop>;
}

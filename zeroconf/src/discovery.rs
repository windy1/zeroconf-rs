use super::Result;
use std::any::Any;
use std::sync::Arc;

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
}

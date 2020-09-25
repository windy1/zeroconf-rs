use std::any::Any;
use std::sync::Arc;

/// Callback invoked from [`MdnsService`] once it has successfully registered.
///
/// # Arguments
/// `service` - The service information that was registered
/// `context` - The optional user context passed through using [`MdnsService::set_context()`]
///
/// [`MdnsService`]: struct.MdnsService.html
/// [`MdnsService::set_context()`]: struct.MdnsService.html#method.set_context
pub type ServiceRegisteredCallback =
    dyn Fn(Result<ServiceRegistration, super::error::Error>, Option<Arc<dyn Any>>);

/// Represents a registration event for a [`MdnsService`].
///
/// [`MdnsService`]: struct.MdnsService.html
#[derive(Builder, BuilderDelegate, Debug, Getters)]
pub struct ServiceRegistration {
    name: String,
    kind: String,
    domain: String,
}

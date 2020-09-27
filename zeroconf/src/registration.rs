use std::any::Any;
use std::sync::Arc;

/// Callback invoked from [`MdnsService`] once it has successfully registered.
///
/// # Arguments
/// * `service` - The service information that was registered
/// * `context` - The optional user context passed through
///
/// [`MdnsService`]: type.MdnsService.html
pub type ServiceRegisteredCallback =
    dyn Fn(Result<ServiceRegistration, super::error::Error>, Option<Arc<dyn Any>>);

/// Represents a registration event for a [`MdnsService`].
///
/// [`MdnsService`]: type.MdnsService.html
#[derive(Builder, BuilderDelegate, Debug, Getters, Clone, Default, PartialEq, Eq)]
pub struct ServiceRegistration {
    name: String,
    kind: String,
    domain: String,
}

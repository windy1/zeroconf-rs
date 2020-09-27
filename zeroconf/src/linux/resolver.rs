//! Rust friendly `AvahiServiceResolver` wrappers/helpers

use super::client::ManagedAvahiClient;
use crate::Result;
use avahi_sys::{
    avahi_service_resolver_free, avahi_service_resolver_new, AvahiIfIndex, AvahiLookupFlags,
    AvahiProtocol, AvahiServiceResolver, AvahiServiceResolverCallback,
};
use libc::{c_char, c_void};
use std::collections::HashMap;

/// Wraps the `AvahiServiceResolver` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiServiceResolver` when
/// `ManagedAvahiServiceResolver::new()` is invoked and calls the Avahi function responsible for
/// freeing the client on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiServiceResolver(*mut AvahiServiceResolver);

impl ManagedAvahiServiceResolver {
    /// Intializes the underlying `*mut AvahiServiceResolver` and verifies it was created;
    /// returning `Err(String)` if unsuccessful.
    pub fn new(
        ManagedAvahiServiceResolverParams {
            client,
            interface,
            protocol,
            name,
            kind,
            domain,
            aprotocol,
            flags,
            callback,
            userdata,
        }: ManagedAvahiServiceResolverParams,
    ) -> Result<Self> {
        let resolver = unsafe {
            avahi_service_resolver_new(
                client.0, interface, protocol, name, kind, domain, aprotocol, flags, callback,
                userdata,
            )
        };

        if resolver.is_null() {
            Err("could not initialize AvahiServiceResolver".into())
        } else {
            Ok(Self(resolver))
        }
    }
}

impl Drop for ManagedAvahiServiceResolver {
    fn drop(&mut self) {
        unsafe { avahi_service_resolver_free(self.0) };
    }
}

/// Holds parameters for initializing a new `ManagedAvahiServiceResolver` with
/// `ManagedAvahiServiceResolver::new()`.
///
/// See [`avahi_service_resolver_new()`] for more information about these parameters.
///
/// [`avahi_service_resolver_new()`]: https://avahi.org/doxygen/html/lookup_8h.html#a904611a4134ceb5919f6bb637df84124
#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiServiceResolverParams<'a> {
    client: &'a ManagedAvahiClient,
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    aprotocol: AvahiProtocol,
    flags: AvahiLookupFlags,
    callback: AvahiServiceResolverCallback,
    userdata: *mut c_void,
}

#[derive(Default, Debug)]
pub(crate) struct ServiceResolverSet {
    resolvers: HashMap<*mut AvahiServiceResolver, ManagedAvahiServiceResolver>,
}

impl ServiceResolverSet {
    pub fn insert(&mut self, resolver: ManagedAvahiServiceResolver) {
        self.resolvers.insert(resolver.0, resolver);
    }

    pub fn remove_raw(&mut self, raw: *mut AvahiServiceResolver) {
        self.resolvers.remove(&raw);
    }
}

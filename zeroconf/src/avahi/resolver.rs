//! Rust friendly `AvahiServiceResolver` wrappers/helpers

use crate::Result;
use avahi_sys::{
    avahi_service_resolver_free, avahi_service_resolver_new, AvahiIfIndex, AvahiLookupFlags,
    AvahiProtocol, AvahiServiceResolver, AvahiServiceResolverCallback,
};
use libc::{c_char, c_void};
use std::{collections::HashMap, sync::Arc};

use super::client::ManagedAvahiClient;

/// Wraps the `AvahiServiceResolver` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiServiceResolver` when
/// `ManagedAvahiServiceResolver::new()` is invoked and calls the Avahi function responsible for
/// freeing the client on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiServiceResolver {
    inner: *mut AvahiServiceResolver,
    _client: Arc<ManagedAvahiClient>,
}

impl ManagedAvahiServiceResolver {
    /// Initializes the underlying `*mut AvahiServiceResolver` and verifies it was created;
    /// returning `Err(String)` if unsuccessful.
    ///
    /// # Safety
    /// This function is unsafe because of the raw pointer dereference.
    pub unsafe fn new(
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
        let inner = unsafe {
            avahi_service_resolver_new(
                client.inner,
                interface,
                protocol,
                name,
                kind,
                domain,
                aprotocol,
                flags,
                callback,
                userdata,
            )
        };

        if inner.is_null() {
            Err("could not initialize AvahiServiceResolver".into())
        } else {
            Ok(Self {
                inner,
                _client: client,
            })
        }
    }
}

impl Drop for ManagedAvahiServiceResolver {
    fn drop(&mut self) {
        unsafe { avahi_service_resolver_free(self.inner) };
    }
}

unsafe impl Send for ManagedAvahiServiceResolver {}
unsafe impl Sync for ManagedAvahiServiceResolver {}

/// Holds parameters for initializing a new `ManagedAvahiServiceResolver` with
/// `ManagedAvahiServiceResolver::new()`.
///
/// See [`avahi_service_resolver_new()`] for more information about these parameters.
///
/// [`avahi_service_resolver_new()`]: https://avahi.org/doxygen/html/lookup_8h.html#a904611a4134ceb5919f6bb637df84124
#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiServiceResolverParams {
    client: Arc<ManagedAvahiClient>,
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
        self.resolvers.insert(resolver.inner, resolver);
    }

    pub fn remove_raw(&mut self, raw: *mut AvahiServiceResolver) {
        self.resolvers.remove(&raw);
    }
}

use super::client::ManagedAvahiClient;
use avahi_sys::{
    avahi_service_resolver_free, avahi_service_resolver_new, AvahiIfIndex, AvahiLookupFlags,
    AvahiProtocol, AvahiServiceResolver, AvahiServiceResolverCallback,
};
use libc::{c_char, c_void};
use std::collections::HashMap;
use std::ptr;

#[derive(Debug)]
pub struct ManagedAvahiServiceResolver {
    resolver: *mut AvahiServiceResolver,
}

impl ManagedAvahiServiceResolver {
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
    ) -> Result<Self, String> {
        let resolver = unsafe {
            avahi_service_resolver_new(
                client.client,
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

        if resolver == ptr::null_mut() {
            Err("could not initialize AvahiServiceResolver".to_string())
        } else {
            Ok(Self { resolver })
        }
    }
}

impl Drop for ManagedAvahiServiceResolver {
    fn drop(&mut self) {
        unsafe { avahi_service_resolver_free(self.resolver) };
    }
}

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
pub struct ServiceResolverSet {
    resolvers: HashMap<*mut AvahiServiceResolver, ManagedAvahiServiceResolver>,
}

impl ServiceResolverSet {
    pub fn insert(&mut self, resolver: ManagedAvahiServiceResolver) {
        self.resolvers.insert(resolver.resolver, resolver);
    }

    pub fn remove_raw(&mut self, raw: *mut AvahiServiceResolver) {
        self.resolvers.remove(&raw);
    }
}

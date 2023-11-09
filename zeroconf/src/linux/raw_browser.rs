//! Rust friendly `AvahiServiceBrowser` wrappers/helpers

use crate::Result;
use avahi_sys::{
    avahi_service_browser_free, avahi_service_browser_get_client, avahi_service_browser_new,
    AvahiClient, AvahiIfIndex, AvahiLookupFlags, AvahiProtocol, AvahiServiceBrowser,
    AvahiServiceBrowserCallback,
};
use libc::{c_char, c_void};

/// Wraps the `AvahiServiceBrowser` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiServiceBrowser` when `ManagedAvahiServiceBrowser::new()`
/// is invoked and calls the Avahi function responsible for freeing the client on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiServiceBrowser {
    inner: *mut AvahiServiceBrowser,
}

impl ManagedAvahiServiceBrowser {
    /// Initializes the underlying `*mut AvahiClient` and verifies it was created; returning
    /// `Err(String)` if unsuccessful.
    pub fn new(
        ManagedAvahiServiceBrowserParams {
            client,
            interface,
            protocol,
            kind,
            domain,
            flags,
            callback,
            userdata,
        }: ManagedAvahiServiceBrowserParams,
    ) -> Result<Self> {
        let inner = unsafe {
            avahi_service_browser_new(
                client, interface, protocol, kind, domain, flags, callback, userdata,
            )
        };

        if inner.is_null() {
            Err("could not initialize Avahi service browser".into())
        } else {
            Ok(Self { inner })
        }
    }

    /// Returns the underlying `*mut AvahiServiceBrowser`.
    ///
    /// # Safety
    /// This function leaks the internal raw pointer, useful for accessing within callbacks where
    /// you are sure the pointer is still valid.
    pub unsafe fn get_client(&self) -> *mut AvahiClient {
        avahi_service_browser_get_client(self.inner)
    }
}

impl Drop for ManagedAvahiServiceBrowser {
    fn drop(&mut self) {
        unsafe { avahi_service_browser_free(self.inner) };
    }
}

/// Holds parameters for initializing a new `ManagedAvahiServiceBrowser` with
/// `ManagedAvahiServiceBrowser::new()`.
///
/// See [`avahi_service_browser_new()`] for more information about these parameters.
///
/// [`avahi_service_browser_new()`]: https://avahi.org/doxygen/html/lookup_8h.html#a52d55a5156a7943012d03e6700880d2b
#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiServiceBrowserParams {
    client: *mut AvahiClient,
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    kind: *const c_char,
    domain: *const c_char,
    flags: AvahiLookupFlags,
    callback: AvahiServiceBrowserCallback,
    userdata: *mut c_void,
}

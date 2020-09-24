use super::client::ManagedAvahiClient;
use avahi_sys::{
    avahi_service_browser_free, avahi_service_browser_new, AvahiIfIndex, AvahiLookupFlags,
    AvahiProtocol, AvahiServiceBrowser, AvahiServiceBrowserCallback,
};
use libc::{c_char, c_void};
use std::ptr;

#[derive(Debug)]
pub struct ManagedAvahiServiceBrowser {
    browser: *mut AvahiServiceBrowser,
}

impl ManagedAvahiServiceBrowser {
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
    ) -> Result<Self, String> {
        let browser = unsafe {
            avahi_service_browser_new(
                client.client,
                interface,
                protocol,
                kind,
                domain,
                flags,
                callback,
                userdata,
            )
        };

        if browser == ptr::null_mut() {
            Err("could not initialize Avahi service browser".to_string())
        } else {
            Ok(Self { browser })
        }
    }
}

impl Drop for ManagedAvahiServiceBrowser {
    fn drop(&mut self) {
        unsafe { avahi_service_browser_free(self.browser) };
    }
}

#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiServiceBrowserParams<'a> {
    client: &'a ManagedAvahiClient,
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    kind: *const c_char,
    domain: *const c_char,
    flags: AvahiLookupFlags,
    callback: AvahiServiceBrowserCallback,
    userdata: *mut c_void,
}

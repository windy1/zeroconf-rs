//! Rust friendly `AvahiClient` wrappers/helpers

use super::avahi_util;
use crate::ffi::c_str;
use crate::Result;
use avahi_sys::{
    avahi_client_free, avahi_client_get_host_name, avahi_client_new, avahi_simple_poll_get,
    AvahiClient, AvahiClientCallback, AvahiClientFlags, AvahiSimplePoll,
};
use libc::{c_int, c_void};

/// Wraps the `AvahiClient` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiClient` when `ManagedAvahiClient::new()` is invoked and
/// calls the Avahi function responsible for freeing the client on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiClient {
    pub(crate) inner: *mut AvahiClient,
}

impl ManagedAvahiClient {
    /// Initializes the underlying `*mut AvahiClient` and verifies it was created; returning
    /// `Err(String)` if unsuccessful.
    pub fn new(
        ManagedAvahiClientParams {
            poll,
            flags,
            callback,
            userdata,
        }: ManagedAvahiClientParams,
    ) -> Result<Self> {
        let mut err: c_int = 0;

        let inner = unsafe {
            avahi_client_new(
                avahi_simple_poll_get(poll),
                flags,
                callback,
                userdata,
                &mut err,
            )
        };

        if inner.is_null() {
            return Err("could not initialize AvahiClient".into());
        }

        match err {
            0 => Ok(Self { inner }),
            _ => Err(format!(
                "could not initialize AvahiClient: {}",
                avahi_util::get_error(err)
            )
            .into()),
        }
    }

    /// Delegate function for [`avahi_client_get_host_name()`].
    ///
    /// [`avahi_client_get_host_name()`]: https://avahi.org/doxygen/html/client_8h.html#a89378618c3c592a255551c308ba300bf
    pub fn host_name<'a>(&self) -> Result<&'a str> {
        unsafe { get_host_name(self.inner) }
    }
}

impl Drop for ManagedAvahiClient {
    fn drop(&mut self) {
        unsafe { avahi_client_free(self.inner) };
    }
}

/// Holds parameters for initializing a new `ManagedAvahiClient` with `ManagedAvahiClient::new()`.
///
/// See [`avahi_client_new()`] for more information about these parameters.
///
/// [`avahi_client_new()`]: https://avahi.org/doxygen/html/client_8h.html#a07b2a33a3e7cbb18a0eb9d00eade6ae6
#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiClientParams {
    poll: *mut AvahiSimplePoll,
    flags: AvahiClientFlags,
    callback: AvahiClientCallback,
    userdata: *mut c_void,
}

pub(super) unsafe fn get_host_name<'a>(client: *mut AvahiClient) -> Result<&'a str> {
    assert_not_null!(client);
    let host_name = avahi_client_get_host_name(client);

    if !host_name.is_null() {
        Ok(c_str::raw_to_str(host_name))
    } else {
        Err("could not get host name from AvahiClient".into())
    }
}

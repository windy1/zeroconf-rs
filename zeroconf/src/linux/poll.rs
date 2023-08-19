//! Rust friendly `AvahiSimplePoll` wrappers/helpers

use crate::Result;
use crate::{error::Error, linux::avahi_util};
use avahi_sys::{
    avahi_simple_poll_free, avahi_simple_poll_iterate, avahi_simple_poll_loop,
    avahi_simple_poll_new, AvahiSimplePoll,
};
use std::{convert::TryInto, time::Duration};

/// Wraps the `AvahiSimplePoll` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiSimplePoll` when `ManagedAvahiClient::new()` is invoked
/// and calls the Avahi function responsible for freeing the poll on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiSimplePoll(*mut AvahiSimplePoll);

impl ManagedAvahiSimplePoll {
    /// Initializes the underlying `*mut AvahiSimplePoll` and verifies it was created; returning
    /// `Err(String)` if unsuccessful
    pub fn new() -> Result<Self> {
        let poll = unsafe { avahi_simple_poll_new() };
        if poll.is_null() {
            Err("could not initialize AvahiSimplePoll".into())
        } else {
            Ok(Self(poll))
        }
    }

    /// Delegate function for [`avahi_simple_poll_loop()`].
    ///
    /// [`avahi_simple_poll_loop()`]: https://avahi.org/doxygen/html/simple-watch_8h.html#a14b4cb29832e8c3de609d4c4e5611985
    pub fn start_loop(&self) -> Result<()> {
        avahi_util::sys_exec(
            || unsafe { avahi_simple_poll_loop(self.0) },
            "could not start AvahiSimplePoll",
        )
    }

    /// Delegate function for [`avahi_simple_poll_iterate()`].
    ///
    /// [`avahi_simple_poll_iterate()`]: https://avahi.org/doxygen/html/simple-watch_8h.html#ad5b7c9d3b7a6584d609241ee6f472a2e
    pub fn iterate(&self, timeout: Duration) -> Result<()> {
        let sleep_time: i32 = timeout
            .as_millis() // `avahi_simple_poll_iterate()` expects `sleep_time` in msecs.
            .try_into() // `avahi_simple_poll_iterate()` expects `sleep_time` as an i32.
            .unwrap_or(i32::MAX); // if converting to an i32 overflows, just use the largest number we can.

        // Returns -1 on error, 0 on success and 1 if a quit request has been scheduled
        match unsafe { avahi_simple_poll_iterate(self.0, sleep_time) } {
            0 | 1 => Ok(()),
            -1 => Err(Error::from(
                "avahi_simple_poll_iterate(..) threw an error result",
            )),
            _ => Err(Error::from(
                "avahi_simple_poll_iterate(..) returned an unknown result",
            )),
        }
    }

    pub(super) fn inner(&self) -> *mut AvahiSimplePoll {
        self.0
    }
}

impl Drop for ManagedAvahiSimplePoll {
    fn drop(&mut self) {
        unsafe { avahi_simple_poll_free(self.0) };
    }
}

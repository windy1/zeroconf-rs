//! Rust friendly `AvahiSimplePoll` wrappers/helpers

use crate::Result;
use avahi_sys::{
    avahi_simple_poll_free, avahi_simple_poll_iterate, avahi_simple_poll_loop,
    avahi_simple_poll_new, AvahiSimplePoll, AvahiPoll
};


/// Platform specific trait providing a polling service.
///
/// The service provides the means for waiting asynchronously for events like
/// ready file descriptors. Avahi does this by the means of the `AvahiPoll`
/// abstraction, Bonjour apparently just polls a single file descriptor.
///
/// User code should treat this as an abstract trait, except when different
/// means for polling should be provided (e.g. integration with some async
/// framework) , in this case the user should provide an implementation for each
/// platform she cares about and use it with
/// `Service::register_with_poll` for example in order to integrate with some
/// other event loop implementation or async framework. This type should then
/// also implement the platform independent `TEventLoop` trait.
pub trait TPoll {
    fn as_avahi_poll(&self) -> *const AvahiPoll;
    // fn as_avahi_poll_mut(&mut self) -> *mut AvahiPoll;
}

/// Make a type suitable for `Service::register`.
///
/// Any type that works without any external dependencies should be able to
/// implement this trait and thus makes the simple `Service::register`
/// available to users.
pub trait TNewPoll: TPoll {
    fn new() -> Result<Self>
        where Self: std::marker::Sized;
}

/// Wraps the `AvahiSimplePoll` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiSimplePoll` when `ManagedAvahiClient::new()` is invoked
/// and calls the Avahi function responsible for freeing the poll on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiSimplePoll(pub(super) *mut AvahiSimplePoll);

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
        avahi!(
            avahi_simple_poll_loop(self.0),
            "could not start AvahiSimplePoll"
        )
    }

    /// Delegate function for [`avahi_simple_poll_iterate()`].
    ///
    /// [`avahi_simple_poll_iterate()`]: https://avahi.org/doxygen/html/simple-watch_8h.html#ad5b7c9d3b7a6584d609241ee6f472a2e
    pub fn iterate(&self, sleep_time: i32) {
        unsafe { avahi_simple_poll_iterate(self.0, sleep_time) };
    }
}

impl TPoll for ManagedAvahiSimplePoll {
    fn as_avahi_poll(&self) -> *const AvahiPoll {
        self.0 as *const AvahiPoll
    }
    // fn as_avahi_poll_mut(mut& self) -> *mut AvahiPoll {
    //     self.0 as *mut AvahiPoll
    // }
}

impl TNewPoll for ManagedAvahiSimplePoll {
    fn new() -> Result<Self> {
        ManagedAvahiSimplePoll::new()
    }
}

impl Drop for ManagedAvahiSimplePoll {
    fn drop(&mut self) {
        unsafe { avahi_simple_poll_free(self.0) };
    }
}

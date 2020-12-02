//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::poll::{
    ManagedAvahiSimplePoll,
    TPoll,
    TNewPoll,
};
use crate::event_loop::TEventLoop;
use crate::Result;
use std::sync::Arc;
use std::time::Duration;

use avahi_sys::AvahiPoll;

#[derive(new,Clone)]
pub struct AvahiEventLoop {
    poll: Arc<ManagedAvahiSimplePoll>,
}

impl TEventLoop for AvahiEventLoop {
    /// Polls for new events.
    ///
    /// Internally calls `ManagedAvahiSimplePoll::iterate(0)`, the `timeout` parameter does not
    /// currently do anything in the Avahi implementation.
    fn poll(&self, _timeout: Duration) -> Result<()> {
        self.poll.iterate(0);
        Ok(())
    }
}

impl TPoll for AvahiEventLoop {
    fn as_avahi_poll(&self) -> *const AvahiPoll {
        self.poll.as_avahi_poll()
    }
}

impl TNewPoll for AvahiEventLoop {
    fn new() -> Result<Self> {
        Ok(AvahiEventLoop {
            poll: Arc::new(ManagedAvahiSimplePoll::new()?),
        }
        )
    }
}

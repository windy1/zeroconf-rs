//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::poll::ManagedAvahiSimplePoll;
use crate::event_loop::TEventLoop;
use crate::Result;
use std::sync::Arc;
use std::time::Duration;

#[derive(new)]
pub struct AvahiEventLoop {
    poll: Arc<ManagedAvahiSimplePoll>,
}

impl TEventLoop for AvahiEventLoop {
    /// Polls for new events.
    ///
    /// Internally calls `ManagedAvahiSimplePoll::iterate(..)`.  
    /// In systems where the C implementation of `poll(.., timeout)`
    /// does not respect the `timeout` parameter, the `timeout` passed
    /// here will have no effect -- ie will return immediately.
    fn poll(&self, timeout: Duration) -> Result<()> {
        self.poll.iterate(timeout)
    }
}

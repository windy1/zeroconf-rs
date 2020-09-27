//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::poll::ManagedAvahiSimplePoll;
use crate::Result;
use std::sync::Arc;
use std::time::Duration;

/// A handle on the underlying Avahi implementation to poll the event loop. Typically `poll()` is
/// called in a loop to keep a [`MdnsService`] or [`MdnsBrowser`] running.
///
/// [`MdnsService`]: ../../type.MdnsService.html
/// [`MdnsBrowser`]: ../../type.MdnsBrowser.html
#[derive(new)]
pub struct AvahiEventLoop {
    poll: Arc<ManagedAvahiSimplePoll>,
}

impl AvahiEventLoop {
    /// Polls for new events.
    ///
    /// Internally calls `ManagedAvahiSimplePoll::iterate(0)`, the `timeout` parameter does not
    /// currently do anything in the Avahi implementation.
    pub fn poll(&self, _timeout: Duration) -> Result<()> {
        self.poll.iterate(0);
        Ok(())
    }
}

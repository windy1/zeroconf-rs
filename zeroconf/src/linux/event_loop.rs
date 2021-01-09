//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::poll::ManagedAvahiSimplePoll;
use crate::event_loop::TEventLoop;
use crate::Result;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

#[derive(new)]
pub struct AvahiEventLoop<'a> {
    poll: Arc<ManagedAvahiSimplePoll>,
    phantom: PhantomData<&'a ManagedAvahiSimplePoll>,
}

impl<'a> TEventLoop for AvahiEventLoop<'a> {
    /// Polls for new events.
    ///
    /// Internally calls `ManagedAvahiSimplePoll::iterate(0)`, the `timeout` parameter does not
    /// currently do anything in the Avahi implementation.
    fn poll(&self, _timeout: Duration) -> Result<()> {
        self.poll.iterate(0);
        Ok(())
    }
}

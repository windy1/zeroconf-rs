//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::poll::ManagedAvahiSimplePoll;
use crate::event_loop::TEventLoop;
use crate::Result;
use std::convert::TryInto;
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
    /// The `timeout` parameter defines the maximum time to sleep, but it will return earlier if
    /// an event occurs.
    fn poll(&self, timeout: Duration) -> Result<()> {
        self.poll.iterate(timeout.as_millis().try_into().unwrap_or(-1))
    }
}

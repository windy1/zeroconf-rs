use super::poll::ManagedAvahiSimplePoll;
use crate::Result;
use std::sync::Arc;
use std::time::Duration;

#[derive(new)]
pub struct AvahiEventLoop {
    poll: Arc<ManagedAvahiSimplePoll>,
}

impl AvahiEventLoop {
    pub fn poll(&self, timeout: Duration) -> Result<()> {
        self.poll.iterate(0);
        Ok(())
    }
}

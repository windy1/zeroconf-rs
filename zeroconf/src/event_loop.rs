//! Trait definition for cross-platform event loop

use crate::Result;
use std::time::Duration;

/// A handle on the underlying implementation to poll the event loop. Typically, `poll()`
/// is called in a loop to keep a `MdnsService` or `MdnsBrowser` running.
pub trait TEventLoop {
    /// Polls for new events.
    fn poll(&self, timeout: Duration) -> Result<()>;
}

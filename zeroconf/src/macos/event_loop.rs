//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::service_ref::ManagedDNSServiceRef;
use crate::Result;
use std::sync::{Arc, Mutex};

/// A handle on the underlying Bonjour implementation to poll the event loop. Typically, `poll()`
/// is called in a loop to keep a [`MdnsService`] or [`MdnsBrowser`] running.
#[derive(new)]
pub struct BonjourEventLoop {
    service: Arc<Mutex<ManagedDNSServiceRef>>,
}

impl BonjourEventLoop {
    pub fn poll(&self) -> Result<()> {
        self.service.lock().unwrap().process_result()
    }
}

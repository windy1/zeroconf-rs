//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::service_ref::ManagedDNSServiceRef;
use crate::{ffi, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A handle on the underlying Bonjour implementation to poll the event loop. Typically, `poll()`
/// is called in a loop to keep a [`MdnsService`] or [`MdnsBrowser`] running.
#[derive(new)]
pub struct BonjourEventLoop {
    service: Arc<Mutex<ManagedDNSServiceRef>>,
}

impl BonjourEventLoop {
    pub fn poll(&self, timeout: Duration) -> Result<()> {
        let service = self.service.lock().unwrap();
        let select = unsafe { ffi::read_select(service.sock_fd(), timeout)? };
        if select > 0 {
            service.process_result()
        } else {
            Ok(())
        }
    }
}

//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::service_ref::ManagedDNSServiceRef;
use crate::event_loop::TEventLoop;
use crate::{Result, ffi};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(new)]
pub struct BonjourEventLoop {
    service: Arc<Mutex<ManagedDNSServiceRef>>,
}

impl TEventLoop for BonjourEventLoop {
    /// Polls for new events.
    ///
    /// Prior to calling `ManagedDNSServiceRef::process_result()`, this function performs a unix
    /// `select()` on the underlying socket with the specified timeout. If the socket contains no
    /// new data, the blocking call is not made.
    fn poll(&self, timeout: Duration) -> Result<()> {
        let service = self
            .service
            .lock()
            .expect("should have been able to obtain lock on service ref");

        let select = unsafe { ffi::bonjour::read_select(service.sock_fd(), timeout)? };

        if select > 0 {
            unsafe { service.process_result() }
        } else {
            Ok(())
        }
    }
}

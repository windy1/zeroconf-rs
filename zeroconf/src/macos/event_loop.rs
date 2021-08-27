//! Event loop for running a `MdnsService` or `MdnsBrowser`.

use super::service_ref::ManagedDNSServiceRef;
use crate::event_loop::TEventLoop;
use crate::{ffi, Result};
use std::time::Duration;

#[derive(new, Debug, Default)]
pub struct BonjourEventLoop {
    service: ManagedDNSServiceRef,
}

impl TEventLoop for BonjourEventLoop {
    /// Polls for new events.
    ///
    /// Prior to calling `ManagedDNSServiceRef::process_result()`, this function performs a unix
    /// `select()` on the underlying socket with the specified timeout. If the socket contains no
    /// new data, the blocking call is not made.
    fn poll(&self, timeout: Duration) -> Result<()> {
        debug!("BonjourEventLoop::poll() timeout = {:?}", timeout);
        let select = unsafe { ffi::macos::read_select(self.service.sock_fd(), timeout)? };
        debug!("BonjourEventLoop::poll() select = {:?}", select);
        if select > 0 {
            self.service.process_result()
        } else {
            Ok(())
        }
    }
}

impl BonjourEventLoop {
    pub fn service(&self) -> &ManagedDNSServiceRef {
        &self.service
    }

    pub fn service_mut(&mut self) -> &mut ManagedDNSServiceRef {
        &mut self.service
    }
}

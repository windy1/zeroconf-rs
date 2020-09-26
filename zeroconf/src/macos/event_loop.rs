use super::service_ref::ManagedDNSServiceRef;
use crate::Result;
use std::sync::{Arc, Mutex};

#[derive(new)]
pub struct BonjourEventLoop {
    service: Arc<Mutex<ManagedDNSServiceRef>>,
}

impl BonjourEventLoop {
    pub fn poll(&self) -> Result<()> {
        self.service.lock().unwrap().process_result()
    }
}

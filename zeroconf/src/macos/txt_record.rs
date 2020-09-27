use crate::Result;
use bonjour_sys::{TXTRecordCreate, TXTRecordDeallocate, TXTRecordRef};
use std::{mem, ptr};

pub struct ManagedTXTRecordRef(TXTRecordRef);

impl ManagedTXTRecordRef {
    pub fn new() -> Self {
        let record = unsafe {
            let mut record: TXTRecordRef = mem::zeroed();
            TXTRecordCreate(&mut record, 0, ptr::null_mut());
            record
        };

        Self(record)
    }

    // pub fn set_value(&mut self) -> Result<()> {}
}

impl Drop for ManagedTXTRecordRef {
    fn drop(&mut self) {
        unsafe { TXTRecordDeallocate(&mut self.0) };
    }
}

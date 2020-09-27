use crate::Result;
use bonjour_sys::{
    TXTRecordContainsKey, TXTRecordCreate, TXTRecordDeallocate, TXTRecordGetBytesPtr,
    TXTRecordGetCount, TXTRecordGetItemAtIndex, TXTRecordGetLength, TXTRecordGetValuePtr,
    TXTRecordRef, TXTRecordRemoveValue, TXTRecordSetValue,
};
use libc::{c_char, c_void};
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

    pub fn get_bytes_ptr(&self) -> *const c_void {
        unsafe { TXTRecordGetBytesPtr(&self.0) }
    }

    pub fn get_length(&self) -> u16 {
        unsafe { TXTRecordGetLength(&self.0) }
    }

    pub fn remove_value(&mut self, key: *const c_char) -> Result<()> {
        let err = unsafe { TXTRecordRemoveValue(&mut self.0, key) };
        if err != 0 {
            Err(format!("could not remove TXT record value (code: {})", err).into())
        } else {
            Ok(())
        }
    }

    pub fn set_value(
        &mut self,
        key: *const c_char,
        value_size: u8,
        value: *const c_void,
    ) -> Result<()> {
        let err = unsafe { TXTRecordSetValue(&mut self.0, key, value_size, value) };
        if err != 0 {
            Err(format!("could not set TXT record value (code: {})", err).into())
        } else {
            Ok(())
        }
    }

    pub fn contains_key(&self, key: *const c_char) -> bool {
        unsafe { TXTRecordContainsKey(self.get_length(), self.get_bytes_ptr(), key) == 1 }
    }

    pub fn get_count(&self) -> u16 {
        unsafe { TXTRecordGetCount(self.get_length(), self.get_bytes_ptr()) }
    }

    pub fn get_item_at_index(
        &self,
        item_index: u16,
        key_buf_len: u16,
        key: *mut c_char,
        value_len: *mut u8,
        value: *mut *const c_void,
    ) -> Result<()> {
        let err = unsafe {
            TXTRecordGetItemAtIndex(
                self.get_length(),
                self.get_bytes_ptr(),
                item_index,
                key_buf_len,
                key,
                value_len,
                value,
            )
        };

        if err != 0 {
            Err(format!("could get item at index for TXT record (code: {})", err).into())
        } else {
            Ok(())
        }
    }

    pub fn get_value_ptr(&self, key: *const c_char, value_len: *mut u8) -> *const c_void {
        unsafe { TXTRecordGetValuePtr(self.get_length(), self.get_bytes_ptr(), key, value_len) }
    }
}

impl Drop for ManagedTXTRecordRef {
    fn drop(&mut self) {
        unsafe { TXTRecordDeallocate(&mut self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::c_str;

    #[test]
    fn set_value_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        record
            .set_value(
                key.as_ptr() as *const c_char,
                value_size,
                value.as_ptr() as *const c_void,
            )
            .unwrap();

        let mut value_len: u8 = 0;
        let result = unsafe {
            c_str::raw_to_str(
                record.get_value_ptr(key.as_ptr() as *const c_char, &mut value_len)
                    as *const c_char,
            )
        };

        assert_eq!(result, "bar");
    }

    #[test]
    fn set_value_null_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value_size = 0;

        record
            .set_value(key.as_ptr() as *const c_char, value_size, ptr::null())
            .unwrap();

        let mut value_len: u8 = 0;
        let result = record.get_value_ptr(key.as_ptr() as *const c_char, &mut value_len);

        assert!(result.is_null());
    }

    #[test]
    fn remove_value_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        record
            .set_value(
                key.as_ptr() as *const c_char,
                value_size,
                value.as_ptr() as *const c_void,
            )
            .unwrap();

        record.remove_value(key.as_ptr() as *const c_char).unwrap();

        let mut value_len: u8 = 0;
        let result = record.get_value_ptr(key.as_ptr() as *const c_char, &mut value_len);

        assert!(result.is_null());
    }

    #[test]
    #[should_panic]
    fn remove_value_missing_key_panics() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        record.remove_value(key.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn contains_key_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        record
            .set_value(
                key.as_ptr() as *const c_char,
                value_size,
                value.as_ptr() as *const c_void,
            )
            .unwrap();

        let no_val = c_string!("baz");

        assert!(record.contains_key(key.as_ptr() as *const c_char));
        assert!(!record.contains_key(no_val.as_ptr() as *const c_char));
    }

    #[test]
    fn get_count_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        record
            .set_value(
                key.as_ptr() as *const c_char,
                value_size,
                value.as_ptr() as *const c_void,
            )
            .unwrap();

        assert_eq!(record.get_count(), 1);
    }

    #[test]
    fn get_item_at_index() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        record
            .set_value(
                key.as_ptr() as *const c_char,
                value_size,
                value.as_ptr() as *const c_void,
            )
            .unwrap();

        let mut value_len: u8 = 0;
        let mut value: *const c_void = ptr::null_mut();

        record
            .get_item_at_index(
                0,
                mem::size_of_val(&key) as u16,
                key.as_ptr() as *mut c_char,
                &mut value_len,
                &mut value,
            )
            .unwrap();

        assert_eq!(unsafe { c_str::raw_to_str(value as *const c_char) }, "bar");
    }
}

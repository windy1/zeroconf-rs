//! Low level interface for interacting with `TXTRecordRef`.

use crate::Result;
use bonjour_sys::{
    TXTRecordContainsKey, TXTRecordCreate, TXTRecordDeallocate, TXTRecordGetBytesPtr,
    TXTRecordGetCount, TXTRecordGetItemAtIndex, TXTRecordGetLength, TXTRecordGetValuePtr,
    TXTRecordRef, TXTRecordRemoveValue, TXTRecordSetValue,
};
use libc::{c_char, c_uchar, c_void};
use std::ffi::CString;
use std::{fmt, mem, ptr};

/// Wraps the `ManagedTXTRecordRef` type from the raw Bonjour bindings.
///
/// `zeroconf::TxtRecord` provides the cross-platform bindings for this functionality.
pub struct ManagedTXTRecordRef(TXTRecordRef);

impl ManagedTXTRecordRef {
    /// Creates a new empty TXT record
    pub fn new() -> Self {
        let record = unsafe {
            let mut record: TXTRecordRef = mem::zeroed();
            TXTRecordCreate(&mut record, 0, ptr::null_mut());
            record
        };

        Self(record)
    }

    /// Delegate function for [`TXTRecordGetBytes()`].
    ///
    /// [`TXTRecordGetBytes()`]: https://developer.apple.com/documentation/dnssd/1804717-txtrecordgetbytesptr?language=objc
    pub fn get_bytes_ptr(&self) -> *const c_void {
        unsafe { TXTRecordGetBytesPtr(&self.0) }
    }

    /// Delegate function for [`TXTRecordGetLength()`].
    ///
    /// [`TXTRecordGetLength()`]: https://developer.apple.com/documentation/dnssd/1804720-txtrecordgetlength?language=objc
    pub fn get_length(&self) -> u16 {
        unsafe { TXTRecordGetLength(&self.0) }
    }

    /// Delegate function for [`TXTRecordRemoveValue()`].
    ///
    /// # Safety
    /// This function is unsafe because it makes no guarantees about `key` and `key` is
    /// dereferenced. `key` is expected to be a non-null `*const c_char`.
    ///
    /// [`TXTRecordRemoveValue()`]: https://developer.apple.com/documentation/dnssd/1804721-txtrecordremovevalue?language=objc
    pub unsafe fn remove_value(&mut self, key: *const c_char) -> Result<()> {
        bonjour!(
            TXTRecordRemoveValue(&mut self.0, key),
            "could not remove TXT record value"
        )
    }

    /// Delegate function for [`TXTRecordSetValue`].
    ///
    /// # Safety
    /// This function is unsafe because it makes no guarantees about it's rew pointer arguments
    /// that are dereferenced.
    ///
    /// [`TXTRecordSetValue`]: https://developer.apple.com/documentation/dnssd/1804723-txtrecordsetvalue?language=objc
    pub unsafe fn set_value(
        &mut self,
        key: *const c_char,
        value_size: u8,
        value: *const c_void,
    ) -> Result<()> {
        bonjour!(
            TXTRecordSetValue(&mut self.0, key, value_size, value),
            "could not set TXT record value"
        )
    }

    /// Delegate function for [`TXTRecordContainsKey`].
    ///
    /// # Safety
    /// This function is unsafe because it makes no guarantees about it's rew pointer arguments
    /// that are dereferenced.
    ///
    /// [`TXTRecordContainsKey`]: https://developer.apple.com/documentation/dnssd/1804705-txtrecordcontainskey?language=objc
    pub unsafe fn contains_key(&self, key: *const c_char) -> bool {
        TXTRecordContainsKey(self.get_length(), self.get_bytes_ptr(), key) == 1
    }

    /// Delegate function for [`TXTRecordGetCount`].
    ///
    /// [`TXTRecordGetCount`]: https://developer.apple.com/documentation/dnssd/1804706-txtrecordgetcount?language=objc
    pub fn get_count(&self) -> u16 {
        _get_count(self.get_length(), self.get_bytes_ptr())
    }

    /// Delegate function for [`TXTRecordGetItemAtIndex`].
    ///
    /// # Safety
    /// This function is unsafe because it makes no guarantees about it's rew pointer arguments
    /// that are dereferenced.
    ///
    /// [`TXTRecordGetItemAtIndex`]: https://developer.apple.com/documentation/dnssd/1804708-txtrecordgetitematindex?language=objc
    pub unsafe fn get_item_at_index(
        &self,
        item_index: u16,
        key_buf_len: u16,
        key: *mut c_char,
        value_len: *mut u8,
        value: *mut *const c_void,
    ) -> Result<()> {
        _get_item_at_index(
            self.get_length(),
            self.get_bytes_ptr(),
            item_index,
            key_buf_len,
            key,
            value_len,
            value,
        )
    }

    /// Delegate function for [`TXTRecordGetValuePtr`].
    ///
    /// # Safety
    /// This function is unsafe because it makes no guarantees about it's rew pointer arguments
    /// that are dereferenced.
    ///
    /// [`TXTRecordGetValuePtr`]: https://developer.apple.com/documentation/dnssd/1804709-txtrecordgetvalueptr?language=objc
    pub unsafe fn get_value_ptr(&self, key: *const c_char, value_len: *mut u8) -> *const c_void {
        TXTRecordGetValuePtr(self.get_length(), self.get_bytes_ptr(), key, value_len)
    }

    pub(crate) unsafe fn clone_raw(raw: *const c_uchar, size: u16) -> Result<Self> {
        let chars = c_string!(alloc(size as usize)).into_raw() as *mut c_uchar;
        ptr::copy(raw, chars, size as usize);
        let chars = CString::from_raw(chars as *mut c_char);

        let mut record = Self::new();

        for i in 0.._get_count(size, chars.as_ptr() as *const c_void) {
            let key = c_string!(alloc(256));
            let mut value_len: u8 = 0;
            let mut value: *const c_void = ptr::null_mut();

            _get_item_at_index(
                size,
                chars.as_ptr() as *const c_void,
                i,
                256,
                key.as_ptr() as *mut c_char,
                &mut value_len,
                &mut value,
            )?;

            record.set_value(key.as_ptr() as *mut c_char, value_len, value)?;
        }

        Ok(record)
    }
}

impl Default for ManagedTXTRecordRef {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ManagedTXTRecordRef {
    fn clone(&self) -> Self {
        unsafe {
            Self::clone_raw(self.get_bytes_ptr() as *const c_uchar, self.get_length()).unwrap()
        }
    }
}

impl Drop for ManagedTXTRecordRef {
    fn drop(&mut self) {
        unsafe { TXTRecordDeallocate(&mut self.0) };
    }
}

impl fmt::Debug for ManagedTXTRecordRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ManagedTXTRecordRef").finish()
    }
}

unsafe impl Send for ManagedTXTRecordRef {}
unsafe impl Sync for ManagedTXTRecordRef {}

fn _get_count(length: u16, data: *const c_void) -> u16 {
    unsafe { TXTRecordGetCount(length, data) }
}

fn _get_item_at_index(
    length: u16,
    data: *const c_void,
    item_index: u16,
    key_buf_len: u16,
    key: *mut c_char,
    value_len: *mut u8,
    value: *mut *const c_void,
) -> Result<()> {
    bonjour!(
        TXTRecordGetItemAtIndex(length, data, item_index, key_buf_len, key, value_len, value),
        "could get item at index for TXT record"
    )
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

        unsafe {
            record
                .set_value(
                    key.as_ptr() as *const c_char,
                    value_size,
                    value.as_ptr() as *const c_void,
                )
                .unwrap();
        }

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

        unsafe {
            record
                .set_value(key.as_ptr() as *const c_char, value_size, ptr::null())
                .unwrap();
        }

        let mut value_len: u8 = 0;
        let result = unsafe { record.get_value_ptr(key.as_ptr() as *const c_char, &mut value_len) };

        assert!(result.is_null());
    }

    #[test]
    fn remove_value_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        unsafe {
            record
                .set_value(
                    key.as_ptr() as *const c_char,
                    value_size,
                    value.as_ptr() as *const c_void,
                )
                .unwrap();

            record.remove_value(key.as_ptr() as *const c_char).unwrap();
        }

        let mut value_len: u8 = 0;
        let result = unsafe { record.get_value_ptr(key.as_ptr() as *const c_char, &mut value_len) };

        assert!(result.is_null());
    }

    #[test]
    #[should_panic]
    fn remove_value_missing_key_panics() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        unsafe {
            record.remove_value(key.as_ptr() as *const c_char).unwrap();
        }
    }

    #[test]
    fn contains_key_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        unsafe {
            record
                .set_value(
                    key.as_ptr() as *const c_char,
                    value_size,
                    value.as_ptr() as *const c_void,
                )
                .unwrap();
        }

        let no_val = c_string!("baz");

        unsafe {
            assert!(record.contains_key(key.as_ptr() as *const c_char));
            assert!(!record.contains_key(no_val.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn get_count_success() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        unsafe {
            record
                .set_value(
                    key.as_ptr() as *const c_char,
                    value_size,
                    value.as_ptr() as *const c_void,
                )
                .unwrap();
        }

        assert_eq!(record.get_count(), 1);
    }

    #[test]
    fn get_item_at_index() {
        let mut record = ManagedTXTRecordRef::new();
        let key = c_string!("foo");
        let value = c_string!("bar");
        let value_size = mem::size_of_val(&value) as u8;

        unsafe {
            record
                .set_value(
                    key.as_ptr() as *const c_char,
                    value_size,
                    value.as_ptr() as *const c_void,
                )
                .unwrap();
        }

        let key = unsafe { c_string!(alloc(256)) };
        let mut value_len: u8 = 0;
        let mut value: *const c_void = ptr::null_mut();

        unsafe {
            record
                .get_item_at_index(
                    0,
                    256,
                    key.as_ptr() as *mut c_char,
                    &mut value_len,
                    &mut value,
                )
                .unwrap();

            let key = c_str::raw_to_str(key.as_ptr() as *const c_char);
            let value = c_str::raw_to_str(value as *const c_char);

            assert_eq!(key, "foo");
            assert_eq!(value, "bar");
        }
    }
}

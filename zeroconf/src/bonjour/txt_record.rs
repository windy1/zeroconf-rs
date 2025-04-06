//! Bonjour implementation for cross-platform TXT record.

use super::txt_record_ref::ManagedTXTRecordRef;
use crate::ffi::c_str;
use crate::txt_record::TTxtRecord;
use crate::Result;
use libc::{c_char, c_void};
use std::ffi::CString;
use std::{ptr, slice};

/// Interface for interfacing with Bonjour's TXT record capabilities.
pub struct BonjourTxtRecord(ManagedTXTRecordRef);

impl TTxtRecord for BonjourTxtRecord {
    fn new() -> Self {
        Self(unsafe { ManagedTXTRecordRef::new() })
    }

    fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        let key = c_string!(key);
        let value = c_string!(value);
        let value_size = value.as_bytes().len();

        unsafe {
            self.0.set_value(
                key.as_ptr() as *const c_char,
                value_size as u8,
                value.as_ptr() as *const c_void,
            )
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let mut value_len: u8 = 0;

        let c_str = c_string!(key);

        let value_raw = unsafe {
            self.0
                .get_value_ptr(c_str.as_ptr() as *const c_char, &mut value_len)
        };

        if value_raw.is_null() {
            None
        } else {
            unsafe { read_value(value_raw, value_len) }.into()
        }
    }

    fn remove(&mut self, key: &str) -> Option<String> {
        let c_str = c_string!(key);
        let prev = self.get(key)?;

        unsafe {
            self.0
                .remove_value(c_str.as_ptr() as *const c_char)
                .expect("could not remove value")
        };

        prev.into()
    }

    fn contains_key(&self, key: &str) -> bool {
        let c_str = c_string!(key);
        unsafe { self.0.contains_key(c_str.as_ptr() as *const c_char) }
    }

    fn len(&self) -> usize {
        unsafe { self.0.get_count() as usize }
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(Iter::new(self))
    }

    fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(Keys(Iter::new(self)))
    }

    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(Values(Iter::new(self)))
    }
}

impl Clone for BonjourTxtRecord {
    fn clone(&self) -> Self {
        Self(unsafe { self.0.clone() })
    }
}

impl BonjourTxtRecord {
    pub(super) fn inner(&self) -> &ManagedTXTRecordRef {
        &self.0
    }
}

impl From<ManagedTXTRecordRef> for BonjourTxtRecord {
    fn from(txt: ManagedTXTRecordRef) -> Self {
        Self(txt)
    }
}

impl PartialEq for BonjourTxtRecord {
    fn eq(&self, other: &Self) -> bool {
        self.to_map() == other.to_map()
    }
}

/// An `Iterator` that allows iteration over a [`BonjourTxtRecord`] similar to a `HashMap`.
#[derive(new)]
pub struct Iter<'a> {
    record: &'a BonjourTxtRecord,
    #[new(default)]
    index: usize,
}

impl Iter<'_> {
    const KEY_LEN: u16 = 256;
}

impl Iterator for Iter<'_> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.record.len() {
            return None;
        }

        let raw_key: CString = unsafe { c_string!(alloc(Iter::KEY_LEN as usize)) };
        let mut value_len: u8 = 0;
        let mut value: *const c_void = ptr::null_mut();

        unsafe {
            self.record
                .0
                .get_item_at_index(
                    self.index as u16,
                    Iter::KEY_LEN,
                    raw_key.as_ptr() as *mut c_char,
                    &mut value_len,
                    &mut value,
                )
                .expect("could not get item at index");
        }

        assert_not_null!(value);

        let key = String::from(c_str::to_str(&raw_key))
            .trim_matches(char::from(0))
            .to_string();

        let value = unsafe { read_value(value, value_len) };

        self.index += 1;

        Some((key, value))
    }
}

/// An `Iterator` that allows iteration over a [`BonjourTxtRecord`]'s keys.
pub struct Keys<'a>(Iter<'a>);

impl Iterator for Keys<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.0)
    }
}

/// An `Iterator` that allows iteration over a [`BonjourTxtRecord`]'s values.
pub struct Values<'a>(Iter<'a>);

impl Iterator for Values<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.1)
    }
}

unsafe fn read_value(value: *const c_void, value_len: u8) -> String {
    let value_len = value_len as usize;
    let value_raw = slice::from_raw_parts(value as *const u8, value_len);
    String::from_utf8(value_raw.to_vec()).expect("could not read value")
}

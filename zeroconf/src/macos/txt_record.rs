//! Bonjour implementation for cross-platform TXT record.

use super::txt_record_ref::ManagedTXTRecordRef;
use crate::ffi::c_str;
use crate::txt_record::TTxtRecord;
use crate::Result;
use libc::{c_char, c_void};
use std::ffi::CString;
use std::{mem, ptr};

/// Interface for interfacting with Bonjour's TXT record capabilities.
#[derive(Clone)]
pub struct BonjourTxtRecord(ManagedTXTRecordRef);

impl TTxtRecord for BonjourTxtRecord {
    fn new() -> Self {
        Self(ManagedTXTRecordRef::new())
    }

    fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        let key = c_string!(key);
        let value = c_string!(value);
        // let value_size = mem::size_of_val(&value) as u8;
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

        let value_raw = unsafe {
            self.0
                .get_value_ptr(c_string!(key).as_ptr() as *const c_char, &mut value_len)
        };

        if value_raw.is_null() {
            None
        } else {
            Some(unsafe { c_str::raw_to_str(value_raw as *const c_char).to_string() })
        }
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        unsafe {
            self.0
                .remove_value(c_string!(key).as_ptr() as *const c_char)
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        unsafe {
            self.0
                .contains_key(c_string!(key).as_ptr() as *const c_char)
        }
    }

    fn len(&self) -> usize {
        self.0.get_count() as usize
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
                .unwrap();
        }

        assert_not_null!(value);

        let key = String::from(raw_key.to_str().unwrap())
            .trim_matches(char::from(0))
            .to_string();

        let value = unsafe { c_str::raw_to_str(value as *const c_char).to_string() };

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

impl<'a> Iterator for Values<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.1)
    }
}

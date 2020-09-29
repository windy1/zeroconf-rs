use super::txt_record_ref::ManagedTXTRecordRef;
use crate::ffi::c_str;
use crate::Result;
use libc::{c_char, c_void};
use std::collections::HashMap;
use std::ffi::CString;
use std::{mem, ptr};

/// Interface for interfacting with Bonjour's TXT record capabilities.
#[derive(Debug)]
pub struct BonjourTxtRecord(pub(crate) ManagedTXTRecordRef);

impl BonjourTxtRecord {
    /// Constructs a new TXT recoord
    pub fn new() -> Self {
        Self(ManagedTXTRecordRef::new())
    }

    /// Inserts the specified value at the specified key.
    pub fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        let key = c_string!(key);
        let value = c_string!(value);
        let value_size = mem::size_of_val(&value) as u8;
        unsafe {
            self.0.set_value(
                key.as_ptr() as *const c_char,
                value_size,
                value.as_ptr() as *const c_void,
            )
        }
    }

    /// Returns the value at the specified key or `None` if no such key exists.
    ///
    /// This function returns a owned `String` because there are no guarantees that the
    /// implementation provides access to the underlying value pointer.
    pub fn get(&self, key: &str) -> Option<String> {
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

    /// Removes the value at the specified key. Returns `Err` if no such key exists.
    pub fn remove(&mut self, key: &str) -> Result<()> {
        unsafe {
            self.0
                .remove_value(c_string!(key).as_ptr() as *const c_char)
        }
    }

    /// Returns true if the TXT record contains the specified key.
    pub fn contains_key(&self, key: &str) -> bool {
        unsafe {
            self.0
                .contains_key(c_string!(key).as_ptr() as *const c_char)
        }
    }

    /// Returns the amount of entries in the TXT record.
    pub fn len(&self) -> usize {
        self.0.get_count() as usize
    }

    /// Returns a new `txt_record::Iter` for iterating over the record as you would a `HashMap`.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(Iter::new(self))
    }

    /// Returns a new `txt_record::Iter` over the records keys.
    pub fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(Keys(Iter::new(self)))
    }

    /// Returns a new `txt_record::Iter` over the records values.
    pub fn values<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(Values(Iter::new(self)))
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

impl<'a> Iterator for Iter<'a> {
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

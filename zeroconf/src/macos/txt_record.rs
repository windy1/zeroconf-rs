use super::txt_record_ref::ManagedTXTRecordRef;
use crate::ffi::c_str;
use crate::Result;
use libc::{c_char, c_void};
use std::collections::HashMap;
use std::ffi::CString;
use std::{mem, ptr};

#[derive(Debug)]
pub struct BonjourTxtRecord(ManagedTXTRecordRef);

impl BonjourTxtRecord {
    pub fn new() -> Self {
        Self(ManagedTXTRecordRef::new())
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        let key = c_string!(key);
        let value = c_string!(value);
        let value_size = mem::size_of_val(&value) as u8;
        self.0.set_value(
            key.as_ptr() as *const c_char,
            value_size,
            value.as_ptr() as *const c_void,
        )
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let mut value_len: u8 = 0;

        let value_raw = self
            .0
            .get_value_ptr(c_string!(key).as_ptr() as *const c_char, &mut value_len);

        if value_raw.is_null() {
            None
        } else {
            Some(unsafe { c_str::raw_to_str(value_raw as *const c_char) })
        }
    }

    pub fn remove(&mut self, key: &str) -> Result<()> {
        self.0
            .remove_value(c_string!(key).as_ptr() as *const c_char)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0
            .contains_key(c_string!(key).as_ptr() as *const c_char)
    }

    pub fn len(&self) -> usize {
        self.0.get_count() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    pub fn keys(&self) -> Keys {
        Keys(Iter::new(self))
    }

    pub fn values(&self) -> Values {
        Values(Iter::new(self))
    }

    pub fn as_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        for (key, value) in self.iter() {
            m.insert(key, value.to_string());
        }
        m
    }
}

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
    type Item = (String, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.record.len() {
            return None;
        }

        let raw_key: CString = unsafe { c_string!(alloc(Iter::KEY_LEN as usize)) };
        let mut value_len: u8 = 0;
        let mut value: *const c_void = ptr::null_mut();

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

        assert_not_null!(value);

        let key = String::from(raw_key.to_str().unwrap())
            .trim_matches(char::from(0))
            .to_string();

        let value = unsafe { c_str::raw_to_str(value as *const c_char) };

        self.index += 1;

        Some((key, value))
    }
}

pub struct Keys<'a>(Iter<'a>);

impl Iterator for Keys<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.0)
    }
}

pub struct Values<'a>(Iter<'a>);

impl<'a> Iterator for Values<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.1)
    }
}

impl From<HashMap<String, String>> for BonjourTxtRecord {
    fn from(map: HashMap<String, String>) -> BonjourTxtRecord {
        let mut record = BonjourTxtRecord::new();
        for (key, value) in map {
            record.insert(&key, &value).unwrap();
        }
        record
    }
}

impl From<HashMap<&str, &str>> for BonjourTxtRecord {
    fn from(map: HashMap<&str, &str>) -> BonjourTxtRecord {
        map.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>()
            .into()
    }
}

impl Clone for BonjourTxtRecord {
    fn clone(&self) -> Self {
        self.as_map().into()
    }
}

impl PartialEq for BonjourTxtRecord {
    fn eq(&self, other: &Self) -> bool {
        self.as_map() == other.as_map()
    }
}

impl Eq for BonjourTxtRecord {}

impl ToString for BonjourTxtRecord {
    fn to_string(&self) -> String {
        unsafe { c_str::raw_to_str(self.0.get_bytes_ptr() as *const c_char).to_string() }
    }
}

use super::string_list::ManagedAvahiStringList;
use crate::Result;
use libc::c_char;
use std::cell::UnsafeCell;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct AvahiTxtRecord(UnsafeCell<ManagedAvahiStringList>);

impl AvahiTxtRecord {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts the specified value at the specified key.
    pub fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        unsafe {
            self.inner().add_pair(
                c_string!(key).as_ptr() as *const c_char,
                c_string!(value).as_ptr() as *const c_char,
            );
        }
        Ok(())
    }

    /// Returns the value at the specified key or `None` if no such key exists.
    ///
    /// This function returns a owned `String` because there are no guarantees that the
    /// implementation provides access to the underlying value pointer.
    pub fn get(&self, key: &str) -> Option<String> {
        unsafe {
            self.inner()
                .find(c_string!(key).as_ptr() as *const c_char)?
                .get_pair()
                .value()
                .as_str()
                .map(|s| s.to_string())
        }
    }

    /// Removes the value at the specified key. Returns `Err` if no such key exists.
    pub fn remove(&mut self, key: &str) -> Result<()> {
        unsafe {
            match self.inner().find(c_string!(key).as_ptr() as *const c_char) {
                None => Err("no such key".into()),
                Some(node) => {
                    node.remove();
                    Ok(())
                }
            }
        }
    }

    /// Returns true if the TXT record contains the specified key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner()
            .find(c_string!(key).as_ptr() as *const c_char)
            .is_some()
    }

    /// Returns the amount of entries in the TXT record.
    pub fn len(&self) -> usize {
        self.inner().length() as usize
    }

    // /// Returns a new `txt_record::Iter` for iterating over the record as you would a `HashMap`.
    // pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
    //     Box::new(Iter::new(self))
    // }
    //
    // /// Returns a new `txt_record::Iter` over the records keys.
    // pub fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
    //     Box::new(Keys(Iter::new(self)))
    // }
    //
    // /// Returns a new `txt_record::Iter` over the records values.
    // pub fn values<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
    //     Box::new(Values(Iter::new(self)))
    // }

    fn inner(&self) -> &mut ManagedAvahiStringList {
        &mut *self.0.get()
    }
}

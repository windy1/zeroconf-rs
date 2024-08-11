//! Avahi implementation for cross-platform TXT record.

use super::string_list::{AvahiStringListNode, ManagedAvahiStringList};
use crate::txt_record::TTxtRecord;
use crate::Result;
use libc::c_char;
use std::cell::UnsafeCell;

pub struct AvahiTxtRecord(UnsafeCell<ManagedAvahiStringList>);

impl TTxtRecord for AvahiTxtRecord {
    fn new() -> Self {
        Self(UnsafeCell::new(unsafe { ManagedAvahiStringList::new() }))
    }

    fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        let c_key = c_string!(key);
        let c_value = c_string!(value);

        unsafe {
            self.inner_mut().add_pair(
                c_key.as_ptr() as *const c_char,
                c_value.as_ptr() as *const c_char,
            );
        }
        Ok(())
    }

    fn get(&self, key: &str) -> Option<String> {
        let c_str = c_string!(key);
        unsafe {
            self.inner_mut()
                .find(c_str.as_ptr() as *const c_char)?
                .get_pair()
                .value()
                .as_str()
                .map(|s| s.to_string())
        }
    }

    fn remove(&mut self, key: &str) -> Option<String> {
        let mut list = unsafe { ManagedAvahiStringList::new() };
        let mut map = self.to_map();
        let prev = map.remove(key);

        for (key, value) in map {
            let c_key = c_string!(key);
            let c_value = c_string!(value);

            unsafe {
                list.add_pair(
                    c_key.as_ptr() as *const c_char,
                    c_value.as_ptr() as *const c_char,
                );
            }
        }

        self.0 = UnsafeCell::new(list);

        prev
    }

    fn contains_key(&self, key: &str) -> bool {
        let c_str = c_string!(key);
        unsafe {
            self.inner_mut()
                .find(c_str.as_ptr() as *const c_char)
                .is_some()
        }
    }

    fn len(&self) -> usize {
        unsafe { self.inner().length() as usize }
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(Iter::new(self.inner_mut().head()))
    }

    fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(Keys(Iter::new(self.inner_mut().head())))
    }

    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(Values(Iter::new(self.inner_mut().head())))
    }
}

impl AvahiTxtRecord {
    #[allow(clippy::mut_from_ref)]
    fn inner_mut(&self) -> &mut ManagedAvahiStringList {
        unsafe { &mut *self.0.get() }
    }

    pub(crate) fn inner(&self) -> &ManagedAvahiStringList {
        unsafe { &*self.0.get() }
    }
}

impl From<ManagedAvahiStringList> for AvahiTxtRecord {
    fn from(list: ManagedAvahiStringList) -> Self {
        Self(UnsafeCell::new(list))
    }
}

impl Clone for AvahiTxtRecord {
    fn clone(&self) -> Self {
        Self::from(unsafe { self.inner().clone() })
    }
}

impl PartialEq for AvahiTxtRecord {
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

pub struct Iter<'a> {
    node: Option<AvahiStringListNode<'a>>,
}

impl<'a> Iter<'a> {
    pub fn new(node: AvahiStringListNode<'a>) -> Self {
        Self { node: Some(node) }
    }
}

impl Iterator for Iter<'_> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let mut n = self.node.take()?;

        if n.list().is_null() {
            return None;
        }

        let pair = unsafe { n.get_pair() };
        self.node = unsafe { n.next() };

        let key = unsafe { pair.key().as_str() }
            .expect("could not key as str")
            .to_string();

        let value = unsafe { pair.value().as_str() }
            .expect("could not get value as str")
            .to_string();

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

impl Iterator for Values<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.1)
    }
}

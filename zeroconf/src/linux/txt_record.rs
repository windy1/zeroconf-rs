//! Avahi implementation for cross-platform TXT record.

use super::string_list::{AvahiStringListNode, ManagedAvahiStringList};
use crate::txt_record::TTxtRecord;
use crate::Result;
use libc::c_char;
use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct AvahiTxtRecord(UnsafeCell<ManagedAvahiStringList>);

impl TTxtRecord for AvahiTxtRecord {
    fn new() -> Self {
        Self(UnsafeCell::default())
    }

    fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        unsafe {
            self.inner_mut().add_pair(
                c_string!(key).as_ptr() as *const c_char,
                c_string!(value).as_ptr() as *const c_char,
            );
        }
        Ok(())
    }

    fn get(&self, key: &str) -> Option<String> {
        unsafe {
            self.inner_mut()
                .find(c_string!(key).as_ptr() as *const c_char)?
                .get_pair()
                .value()
                .as_str()
                .map(|s| s.to_string())
        }
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        let mut list = ManagedAvahiStringList::new();
        let mut map = self.to_map();

        map.remove(key);

        for (key, value) in map {
            unsafe {
                list.add_pair(
                    c_string!(key).as_ptr() as *const c_char,
                    c_string!(value).as_ptr() as *const c_char,
                );
            }
        }

        self.0 = UnsafeCell::new(list);

        Ok(())
    }

    fn contains_key(&self, key: &str) -> bool {
        unsafe {
            self.inner_mut()
                .find(c_string!(key).as_ptr() as *const c_char)
                .is_some()
        }
    }

    fn len(&self) -> usize {
        self.inner().length() as usize
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
        let pair = n.get_pair();
        self.node = n.next();

        Some((
            pair.key().as_str().unwrap().to_string(),
            pair.value().as_str().unwrap().to_string(),
        ))
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

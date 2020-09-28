use crate::ffi::c_str;
use avahi_sys::{
    avahi_free, avahi_string_list_add_pair, avahi_string_list_copy, avahi_string_list_equal,
    avahi_string_list_find, avahi_string_list_free, avahi_string_list_get_pair,
    avahi_string_list_length, avahi_string_list_new, avahi_string_list_to_string, AvahiStringList,
};
use libc::{c_char, c_void};
use std::ptr;

#[derive(Debug)]
pub struct ManagedAvahiStringList(*mut AvahiStringList);

impl ManagedAvahiStringList {
    pub fn new() -> Self {
        Self(unsafe { avahi_string_list_new(ptr::null()) })
    }

    pub unsafe fn add_pair(&mut self, key: *const c_char, value: *const c_char) {
        self.0 = avahi_string_list_add_pair(self.0, key, value);
    }

    pub unsafe fn find(&mut self, key: *const c_char) -> Option<AvahiStringListNode> {
        let node = avahi_string_list_find(self.0, key);
        if !node.is_null() {
            Some(AvahiStringListNode::new(node))
        } else {
            None
        }
    }

    pub fn length(&self) -> u32 {
        unsafe { avahi_string_list_length(self.0) }
    }

    pub fn to_string(&self) -> AvahiString {
        unsafe { avahi_string_list_to_string(self.0).into() }
    }
}

impl Clone for ManagedAvahiStringList {
    fn clone(&self) -> Self {
        Self(unsafe { avahi_string_list_copy(self.0) })
    }
}

impl PartialEq for ManagedAvahiStringList {
    fn eq(&self, other: &Self) -> bool {
        unsafe { avahi_string_list_equal(self.0, other.0) == 1 }
    }
}

impl Eq for ManagedAvahiStringList {}

impl Default for ManagedAvahiStringList {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ManagedAvahiStringList {
    fn drop(&mut self) {
        unsafe { avahi_string_list_free(self.0) };
    }
}

#[derive(new)]
pub struct AvahiStringListNode(*mut AvahiStringList);

impl AvahiStringListNode {
    pub fn get_pair(&mut self) -> AvahiPair {
        let mut key: *mut c_char = ptr::null_mut();
        let mut value: *mut c_char = ptr::null_mut();
        let mut value_size: usize = 0;

        unsafe {
            avahi_string_list_get_pair(self.0, &mut key, &mut value, &mut value_size);
        }

        AvahiPair::new(key.into(), value.into(), value_size)
    }
}

#[derive(new, Getters)]
pub struct AvahiPair {
    key: AvahiString,
    value: AvahiString,
    value_size: usize,
}

#[derive(new)]
pub struct AvahiString(*mut c_char);

impl AvahiString {
    pub fn as_str(&self) -> &str {
        unsafe { c_str::raw_to_str(self.0) }
    }
}

impl From<*mut c_char> for AvahiString {
    fn from(s: *mut c_char) -> Self {
        Self::new(s)
    }
}

impl ToString for AvahiString {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Drop for AvahiString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { avahi_free(self.0 as *mut c_void) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get_pair_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key1 = c_string!("foo");
        let value1 = c_string!("bar");

        list.add_pair(
            key1.as_ptr() as *const c_char,
            value1.as_ptr() as *const c_char,
        );

        let key2 = c_string!("hello");
        let value2 = c_string!("world");

        list.add_pair(
            key2.as_ptr() as *const c_char,
            value2.as_ptr() as *const c_char,
        );

        let pair1 = list
            .find(key1.as_ptr() as *const c_char)
            .unwrap()
            .get_pair();

        let pair2 = list
            .find(key2.as_ptr() as *const c_char)
            .unwrap()
            .get_pair();

        assert_eq!(pair1.key().as_str(), "foo");
        assert_eq!(pair1.value().as_str(), "bar");
        assert_eq!(pair2.key().as_str(), "hello");
        assert_eq!(pair2.value().as_str(), "world");
    }

    #[test]
    fn add_pair_replaces_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        list.add_pair(
            key.as_ptr() as *const c_char,
            value.as_ptr() as *const c_char,
        );

        let pair = list.find(key.as_ptr() as *const c_char).unwrap().get_pair();

        assert_eq!(pair.value().as_str(), "bar");

        let value = c_string!("baz");

        list.add_pair(
            key.as_ptr() as *const c_char,
            value.as_ptr() as *const c_char,
        );

        let pair = list.find(key.as_ptr() as *const c_char).unwrap().get_pair();

        assert_eq!(pair.value().as_str(), "baz");
    }

    #[test]
    fn length_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        list.add_pair(
            key.as_ptr() as *const c_char,
            value.as_ptr() as *const c_char,
        );

        assert_eq!(list.length(), 1);
    }

    #[test]
    fn to_string_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        list.add_pair(
            key.as_ptr() as *const c_char,
            value.as_ptr() as *const c_char,
        );

        assert_eq!(list.to_string().as_str(), "\"foo=bar\"");
    }

    #[test]
    fn equals_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        list.add_pair(
            key.as_ptr() as *const c_char,
            value.as_ptr() as *const c_char,
        );

        assert_eq!(list.clone(), list);
    }
}

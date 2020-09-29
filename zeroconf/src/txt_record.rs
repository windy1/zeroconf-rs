//! TxtRecord utilities common to all platforms

use crate::{Result, TxtRecord};
use std::collections::HashMap;

/// Interface for interacting with underlying mDNS implementation TXT record capabilities
pub trait TTxtRecord {
    /// Constructs a new TXT record
    fn new() -> Self;

    /// Inserts the specified value at the specified key.
    fn insert(&mut self, key: &str, value: &str) -> Result<()>;

    /// Returns the value at the specified key or `None` if no such key exists.
    ///
    /// This function returns a owned `String` because there are no guarantees that the
    /// implementation provides access to the underlying value pointer.
    fn get(&self, key: &str) -> Option<String>;

    /// Removes the value at the specified key. Returns `Err` if no such key exists.
    fn remove(&mut self, key: &str) -> Result<()>;

    /// Returns true if the TXT record contains the specified key.
    fn contains_key(&self, key: &str) -> bool;

    /// Returns the amount of entries in the TXT record.
    fn len(&self) -> usize;

    /// Returns a new iterator for iterating over the record as you would a `HashMap`.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a>;

    /// Returns a new iterator over the records keys.
    fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a>;

    /// Returns a new iterator over the records values.
    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a>;

    /// Returns true if there are no entries in the record.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a new `HashMap` with this record's keys and values.
    fn to_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        for (key, value) in self.iter() {
            m.insert(key, value.to_string());
        }
        m
    }
}

impl From<HashMap<String, String>> for TxtRecord {
    fn from(map: HashMap<String, String>) -> TxtRecord {
        let mut record = TxtRecord::new();
        for (key, value) in map {
            record.insert(&key, &value).unwrap();
        }
        record
    }
}

impl From<HashMap<&str, &str>> for TxtRecord {
    fn from(map: HashMap<&str, &str>) -> TxtRecord {
        map.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>()
            .into()
    }
}

impl Clone for TxtRecord {
    fn clone(&self) -> Self {
        self.to_map().into()
    }
}

impl PartialEq for TxtRecord {
    fn eq(&self, other: &Self) -> bool {
        self.to_map() == other.to_map()
    }
}

impl Eq for TxtRecord {}

impl Default for TxtRecord {
    fn default() -> Self {
        Self::new()
    }
}

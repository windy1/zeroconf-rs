//! TxtRecord utilities common to all platforms

use crate::{Result, TxtRecord};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::marker::PhantomData;

/// Interface for interacting with underlying mDNS implementation TXT record capabilities
pub trait TTxtRecord: Clone + PartialEq + Eq + Debug {
    /// Constructs a new TXT record
    fn new() -> Self;

    /// Inserts the specified value at the specified key.
    fn insert(&mut self, key: &str, value: &str) -> Result<()>;

    /// Returns the value at the specified key or `None` if no such key exists.
    ///
    /// This function returns an owned `String` because there are no guarantees that the
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

impl Eq for TxtRecord {}

impl Default for TxtRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for TxtRecord {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.iter() {
            map.serialize_entry(&key, &value)?;
        }
        map.end()
    }
}

#[derive(new)]
struct TxtRecordVisitor {
    marker: PhantomData<fn() -> TxtRecord>,
}

impl<'de> Visitor<'de> for TxtRecordVisitor {
    type Value = TxtRecord;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("map containing TXT record data")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = TxtRecord::new();

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value).unwrap();
        }

        Ok(map)
    }
}

impl<'de> Deserialize<'de> for TxtRecord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TxtRecordVisitor::new())
    }
}

impl Debug for TxtRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TxtRecord")
            .field("data", &self.to_map())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_success() {
        crate::tests::setup();

        let mut txt = TxtRecord::new();
        txt.insert("foo", "bar").unwrap();

        let json = serde_json::to_string(&txt).unwrap();
        let txt_de: TxtRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(txt, txt_de);
    }
}

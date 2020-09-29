//! TxtRecord utilities common to all platforms

use crate::TxtRecord;
use std::collections::HashMap;

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

impl TxtRecord {
    /// Returns true if there are no entries in the record.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a new `HashMap` with this record's keys and values.
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        for (key, value) in self.iter() {
            m.insert(key, value.to_string());
        }
        m
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

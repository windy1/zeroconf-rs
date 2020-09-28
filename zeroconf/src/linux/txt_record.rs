use crate::Result;
use std::collections::{hash_map, HashMap};

#[derive(Debug)]
pub struct AvahiTxtRecord(HashMap<String, String>);

impl AvahiTxtRecord {
    /// Constructs a new TXT record
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts the specified value at the specified key.
    pub fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        self.0.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Returns the value at the specified key or `None` if no such key exists.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    /// Removes the value at the specified key. Returns `Err` if no such key exists.
    pub fn remove(&mut self, key: &str) -> Result<()> {
        match self.0.remove(key) {
            None => Err("no such key in TXT record".into()),
            Some(_) => Ok(()),
        }
    }

    /// Returns true if the TXT record contains the specified key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Returns the amount of entries in the TXT record.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no entries in the record.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

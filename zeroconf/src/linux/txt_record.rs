use crate::Result;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

    /// Returns a new `txt_record::Iter` for iterating over the record as you would a `HashMap`.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (String, &'a str)> + 'a> {
        Box::new(self.0.iter().map(|(k, v)| (k.to_string(), v.as_str())))
    }

    /// Returns a new `txt_record::Iter` over the records keys.
    pub fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(self.0.keys().map(|k| k.to_string()))
    }

    /// Returns a new `txt_record::Iter` over the records values.
    pub fn values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(self.0.values().map(|v| v.as_str()))
    }

    /// Returns a new `HashMap` with this record's keys and values.
    pub fn to_map(&self) -> HashMap<String, String> {
        self.0.clone()
    }
}

impl From<HashMap<String, String>> for AvahiTxtRecord {
    fn from(map: HashMap<String, String>) -> AvahiTxtRecord {
        Self(map)
    }
}

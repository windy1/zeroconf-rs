//! TxtRecord utilities common to all platforms

use crate::TxtRecord;
use std::collections::HashMap;
use std::ops::Index;

impl Index<&str> for TxtRecord {
    type Output = str;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key).unwrap()
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

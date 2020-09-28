use crate::TxtRecord;
use std::ops::Index;

impl Index<&str> for TxtRecord {
    type Output = str;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key).unwrap()
    }
}

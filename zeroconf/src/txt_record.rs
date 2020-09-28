use crate::TxtRecord;
use std::ops::Index;

pub type Iter<'a> = dyn Iterator<Item = (String, &'a str)>;
pub type Keys<'a> = dyn Iterator<Item = String>;
pub type Values<'a> = dyn Iterator<Item = &'a str>;

impl Index<&str> for TxtRecord {
    type Output = str;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key).unwrap()
    }
}

use crate::TxtRecord;
use std::collections::HashMap;

#[test]
fn insert_get_success() {
    super::setup();
    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    assert_eq!(&record["foo"], "bar");
    assert_eq!(record.get("baz"), None);
}

#[test]
fn remove_success() {
    super::setup();
    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    record.remove("foo").unwrap();
    assert!(record.get("foo").is_none());
}

#[test]
fn contains_key_success() {
    super::setup();
    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    assert!(record.contains_key("foo"));
    assert!(!record.contains_key("baz"));
}

#[test]
fn len_success() {
    super::setup();
    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    assert_eq!(record.len(), 1);
}

#[test]
#[ignore]
fn iter_success() {
    super::setup();

    debug!("iter_success()");

    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    record.insert("baz", "qux").unwrap();
    record.insert("hello", "world").unwrap();

    for (key, value) in record.iter() {
        debug!("({:?}, {:?})", key, value);
    }
}

#[test]
#[ignore]
fn keys_success() {
    super::setup();

    debug!("keys_success()");

    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    record.insert("baz", "qux").unwrap();
    record.insert("hello", "world").unwrap();

    for key in record.keys() {
        debug!("{:?}", key);
    }
}

#[test]
#[ignore]
fn values_success() {
    super::setup();

    debug!("values_success()");

    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();
    record.insert("baz", "qux").unwrap();
    record.insert("hello", "world").unwrap();

    for value in record.values() {
        debug!("{:?}", value);
    }
}

#[test]
fn from_hashmap_success() {
    super::setup();

    let mut map = HashMap::new();
    map.insert("foo", "bar");

    let record: TxtRecord = map.into();

    assert_eq!(&record["foo"], "bar");
}

#[test]
fn clone_success() {
    super::setup();

    let mut record = TxtRecord::new();
    record.insert("foo", "bar").unwrap();

    assert_eq!(record.clone(), record);
}

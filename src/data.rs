use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
pub struct StoredValue {
    value: Bytes,
    flags: u16,
    exptime: isize,
    byte_count: usize,
}

impl StoredValue {
    pub fn new() -> Self {
        StoredValue {
            value: Bytes::new(),
            flags: 0,
            exptime: 0,
            byte_count: 0,
        }
    }

    pub fn set_bytes(&mut self, value: Bytes) {
        self.value = value;
    }

    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags;
    }

    pub fn set_exptime(&mut self, exptime: isize) {
        self.exptime = exptime;
    }

    pub fn set_byte_count(&mut self, byte_count: usize) {
        self.byte_count = byte_count;
    }

    pub fn get_byte_count(&mut self) -> usize {
        self.byte_count
    }

    pub fn response_string(self, key: &str) -> String {
        format!(
            "VALUE {} {} {}\r\n{}\r\nEND\r\n",
            key,
            self.flags,
            self.byte_count,
            std::str::from_utf8(&self.value).unwrap(),
        )
    }
}

#[derive(Default)]
pub struct DataStore {
    store: Arc<DashMap<String, StoredValue>>,
}

impl DataStore {
    pub fn new() -> Self {
        DataStore {
            store: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, key: String) -> Option<StoredValue> {
        self.store.get(&key).map(|v| v.clone())
    }

    pub fn set(&self, key: String, value: StoredValue) {
        // let store = Arc::clone(&self.store);
        self.store.insert(key, value);
    }
}

#[test]
fn data_store_new() {
    let data_store = DataStore::new();
    assert_eq!(0, data_store.store.len());
}

#[test]
fn data_store_set() {
    let data_store = DataStore::new();
    let key = String::from("set_test");
    let data = Bytes::from("lets see if this works");
    let value = StoredValue {
        value: data,
        flags: 0,
        exptime: 0,
        byte_count: 22,
    };

    data_store.set(key, value);
    assert_eq!(1, data_store.store.len());
}

#[test]
fn data_store_get() {
    let data_store = DataStore::new();
    let key = String::from("get_test");
    let data = Bytes::from("lets see if this works");
    let value = StoredValue {
        value: data,
        flags: 0,
        exptime: 0,
        byte_count: 22,
    };
    data_store.set(key.clone(), value.clone());

    let get_value = data_store.get(key);
    match get_value {
        Some(result) => {
            assert_eq!(value.value, result.value);
            assert_eq!(value.flags, result.flags);
            assert_eq!(value.exptime, result.exptime);
            assert_eq!(value.byte_count, result.byte_count);
        }
        None => unreachable!(),
    }
}

#[test]
fn data_store_get_invalid_key() {
    let data_store = DataStore::new();
    let key = String::from("get_test");
    let data = Bytes::from("lets see if this works");
    let value = StoredValue {
        value: data,
        flags: 0,
        exptime: 0,
        byte_count: 22,
    };
    data_store.set(key.clone(), value.clone());

    if data_store.get(String::from("not a key")).is_some() {
        panic!()
    }
}

#[test]
fn stored_value_new() {
    let stored_value = StoredValue::new();
    assert_eq!(0, stored_value.value.len());
    assert_eq!(0, stored_value.flags);
    assert_eq!(0, stored_value.exptime);
    assert_eq!(0, stored_value.byte_count);
}

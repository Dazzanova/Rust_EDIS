use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Store {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: String) {
        let mut map = self.data.lock().unwrap();
        map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let map = self.data.lock().unwrap();
        map.get(key).cloned()
    }

    pub fn del(&self, key: &str) -> bool {
        let mut map = self.data.lock().unwrap();
        map.remove(key).is_some()
    }
}

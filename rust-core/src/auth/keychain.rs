use std::collections::HashMap;

pub struct Keychain {
    entries: HashMap<String, String>,
}

impl Keychain {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
    }
}

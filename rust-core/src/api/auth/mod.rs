mod key_manager;

pub use key_manager::*;

use std::collections::HashMap;
use std::sync::RwLock;

pub struct ApiKeyStore {
    keys: RwLock<HashMap<String, String>>,
}

impl ApiKeyStore {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }

    pub fn set(&self, provider: &str, key: &str) {
        let mut keys = self.keys.write().unwrap();
        keys.insert(provider.to_string(), key.to_string());
    }

    pub fn get(&self, provider: &str) -> Option<String> {
        let keys = self.keys.read().unwrap();
        keys.get(provider).cloned()
    }

    pub fn remove(&self, provider: &str) -> Option<String> {
        let mut keys = self.keys.write().unwrap();
        keys.remove(provider)
    }

    pub fn list_providers(&self) -> Vec<String> {
        let keys = self.keys.read().unwrap();
        keys.keys().cloned().collect()
    }
}

impl Default for ApiKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

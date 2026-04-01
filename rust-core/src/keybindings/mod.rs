use std::collections::HashMap;

pub struct KeyBindings {
    bindings: HashMap<String, String>,
}

impl KeyBindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, key: &str, action: &str) {
        self.bindings.insert(key.to_string(), action.to_string());
    }
}

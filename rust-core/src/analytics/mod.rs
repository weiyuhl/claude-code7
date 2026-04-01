pub struct AnalyticsEvent {
    pub name: String,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

impl AnalyticsEvent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            properties: std::collections::HashMap::new(),
        }
    }
}

pub struct Memory {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
}

impl Memory {
    pub fn new(id: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
        }
    }
}

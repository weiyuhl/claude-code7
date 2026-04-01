pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
}

impl Skill {
    pub fn new(name: &str, description: &str, content: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            content: content.to_string(),
        }
    }
}

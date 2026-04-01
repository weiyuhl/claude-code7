pub struct Plugin {
    pub name: String,
    pub version: String,
}

impl Plugin {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

pub struct McpClient {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl McpClient {
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args: Vec::new(),
        }
    }
}

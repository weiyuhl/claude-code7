pub struct LspClient {
    pub language: String,
}

impl LspClient {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
        }
    }
}

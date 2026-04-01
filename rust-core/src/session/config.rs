use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(rename = "api_key")]
    pub api_key: String,
    pub model: String,
    #[serde(rename = "max_tokens")]
    pub max_tokens: usize,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub thinking_enabled: Option<bool>,
    #[serde(default)]
    pub working_directory: Option<String>,
}

impl SessionConfig {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            max_tokens: 4096,
            provider: None,
            base_url: None,
            temperature: None,
            thinking_enabled: None,
            working_directory: None,
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            provider: None,
            base_url: None,
            temperature: None,
            thinking_enabled: Some(true),
            working_directory: None,
        }
    }
}

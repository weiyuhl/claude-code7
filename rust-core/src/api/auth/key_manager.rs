use crate::session::ClaudeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct KeyManager {
    config_dir: PathBuf,
}

impl KeyManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("claude-mobile");

        std::fs::create_dir_all(&config_dir).ok();

        Self { config_dir }
    }

    pub fn get_key_path(&self, provider: &str) -> PathBuf {
        self.config_dir.join(format!("{}_key", provider))
    }

    pub fn save_key(&self, provider: &str, key: &str) -> Result<(), ClaudeError> {
        let key_path = self.get_key_path(provider);
        std::fs::write(&key_path, key)
            .map_err(|e| ClaudeError::IoError {
                path: key_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;
        Ok(())
    }

    pub fn load_key(&self, provider: &str) -> Result<String, ClaudeError> {
        let key_path = self.get_key_path(provider);
        std::fs::read_to_string(&key_path)
            .map_err(|e| ClaudeError::IoError {
                path: key_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })
    }

    pub fn delete_key(&self, provider: &str) -> Result<(), ClaudeError> {
        let key_path = self.get_key_path(provider);
        if key_path.exists() {
            std::fs::remove_file(&key_path)
                .map_err(|e| ClaudeError::IoError {
                    path: key_path.to_string_lossy().to_string(),
                    message: e.to_string(),
                })?;
        }
        Ok(())
    }

    pub fn has_key(&self, provider: &str) -> bool {
        self.get_key_path(provider).exists()
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentials {
    pub provider: String,
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
}

impl ProviderCredentials {
    pub fn new(provider: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            api_key: api_key.into(),
            base_url: None,
            models: Vec::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = models;
        self
    }
}

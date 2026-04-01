mod mod_impl;
mod openrouter;
mod deepseek;
mod siliconflow;
mod common;

pub use mod_impl::*;
pub use openrouter::*;
pub use deepseek::*;
pub use siliconflow::*;
pub use common::*;

use async_trait::async_trait;
use crate::message::Message;
use crate::session::ClaudeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub id: String,
    pub content: String,
    pub model: String,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: Option<String>,
    pub owned_by: Option<String>,
    pub context_length: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub is_available: bool,
    pub total_balance: Option<String>,
    pub currency: Option<String>,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supported_models(&self) -> Vec<&'static str>;

    async fn chat_completion(
        &self,
        model: &str,
        messages: &[&Message],
        max_tokens: usize,
    ) -> Result<ModelResponse, ClaudeError>;

    async fn stream_chat_completion(
        &self,
        model: &str,
        messages: &[&Message],
        max_tokens: usize,
        callback: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, ClaudeError>;

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ClaudeError> {
        Err(ClaudeError::NetworkError {
            message: "list_models not implemented for this provider".to_string(),
        })
    }

    async fn get_balance(&self) -> Result<BalanceInfo, ClaudeError> {
        Err(ClaudeError::NetworkError {
            message: "get_balance not implemented for this provider".to_string(),
        })
    }

    fn get_base_url(&self) -> &str;
    fn get_api_key(&self) -> &str;
    fn box_clone(&self) -> Box<dyn Provider>;
}

impl Clone for Box<dyn Provider> {
    fn clone(&self) -> Box<dyn Provider> {
        self.box_clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub default_model: String,
    #[serde(default)]
    pub supported_models: Vec<String>,
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
}

pub struct ProviderManager {
    providers: std::sync::RwLock<Vec<Box<dyn Provider>>>,
    default_provider: std::sync::RwLock<Option<String>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: std::sync::RwLock::new(Vec::new()),
            default_provider: std::sync::RwLock::new(None),
        }
    }

    pub fn register(&self, provider: Box<dyn Provider>) {
        let mut providers = self.providers.write().unwrap();
        providers.push(provider);
    }

    pub fn get(&self, name: &str) -> Option<Box<dyn Provider>> {
        let providers = self.providers.read().unwrap();
        providers.iter().find(|p| p.name() == name).map(|p| p.box_clone())
    }

    pub fn set_default(&self, name: &str) -> Result<(), ClaudeError> {
        let mut default = self.default_provider.write().unwrap();
        *default = Some(name.to_string());
        Ok(())
    }

    pub fn get_default(&self) -> Option<Box<dyn Provider>> {
        let default = self.default_provider.read().unwrap();
        default.as_ref()?;
        let providers = self.providers.read().unwrap();
        providers.iter().find(|p| Some(p.name()) == default.as_deref()).map(|p| p.box_clone())
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

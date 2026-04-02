use crate::api::providers::{Provider, ModelResponse, Usage, ModelInfo, BalanceInfo};
use crate::message::{Message, normalize_messages_for_api, ApiMessage};
use crate::session::ClaudeError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DeepSeekProvider {
    api_key: String,
    base_url: String,
    client: Client,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

#[derive(Debug, Clone)]
struct RateLimiter {
    #[allow(dead_code)]
    requests_remaining: u32,
    #[allow(dead_code)]
    tokens_remaining: u32,
    #[allow(dead_code)]
    reset_at: std::time::Instant,
}

impl DeepSeekProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://api.deepseek.com".to_string()),
            client: Client::new(),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                requests_remaining: 100,
                tokens_remaining: 100000,
                reset_at: std::time::Instant::now(),
            })),
        }
    }
}

impl Clone for DeepSeekProvider {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            client: self.client.clone(),
            rate_limiter: Arc::clone(&self.rate_limiter),
        }
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    id: String,
    choices: Vec<Choice>,
    model: String,
    usage: UsageResponse,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct UsageResponse {
    prompt_tokens: u64,
    completion_tokens: u64,
    #[allow(dead_code)]
    total_tokens: u64,
}

#[async_trait]
impl Provider for DeepSeekProvider {
    fn name(&self) -> &'static str {
        "deepseek"
    }

    fn supported_models(&self) -> Vec<&'static str> {
        vec![]
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }

    fn get_api_key(&self) -> &str {
        &self.api_key
    }

    async fn chat_completion(
        &self,
        model: &str,
        messages: &[&Message],
        max_tokens: usize,
    ) -> Result<ModelResponse, ClaudeError> {
        let api_messages = normalize_messages_for_api(messages);

        let request = ChatCompletionRequest {
            model,
            messages: api_messages,
            max_tokens: Some(max_tokens),
            temperature: None,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ClaudeError::NetworkError {
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClaudeError::ApiError {
                provider: "deepseek".to_string(),
                message: format!("HTTP {} - {}", status, body),
            });
        }

        let response: DeepSeekResponse = response
            .json()
            .await
            .map_err(|e| ClaudeError::SerializationError {
                message: e.to_string(),
            })?;

        let first_choice = response.choices.first().ok_or_else(|| ClaudeError::ApiError {
            provider: "deepseek".to_string(),
            message: "No choices in response".to_string(),
        })?;

        Ok(ModelResponse {
            id: response.id,
            content: first_choice.message.content.clone().unwrap_or_default(),
            thinking: first_choice.message.reasoning_content.clone(),
            model: response.model,
            usage: Usage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
            stop_reason: first_choice.finish_reason.clone(),
        })
    }

    async fn stream_chat_completion(
        &self,
        model: &str,
        messages: &[&Message],
        max_tokens: usize,
        callback: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, ClaudeError> {
        let api_messages = normalize_messages_for_api(messages);

        let request = ChatCompletionRequest {
            model,
            messages: api_messages,
            max_tokens: Some(max_tokens),
            temperature: None,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&request)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ClaudeError::NetworkError {
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClaudeError::ApiError {
                provider: "deepseek".to_string(),
                message: format!("HTTP {} - {}", status, body),
            });
        }

        let mut full_response = String::new();

        use futures::StreamExt;

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    buffer.extend_from_slice(&bytes);
                    
                    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line = String::from_utf8_lossy(&buffer[..pos]).trim().to_string();
                        buffer.drain(..pos + 1);

                        if line.starts_with("data: ") {
                            let data = line[6..].trim();
                            if data == "[DONE]" {
                                return Ok(full_response);
                            }
                            
                            if let Ok(event) = serde_json::from_str::<SSEEvent>(data) {
                                if let Some(choices) = &event.choices {
                                    if let Some(first_choice) = choices.get(0) {
                                        if let Some(finish_reason) = &first_choice.finish_reason {
                                            if finish_reason == "error" || finish_reason == "length" {
                                                let error_msg = first_choice.delta.content.as_deref()
                                                    .unwrap_or_else(|| {
                                                        static FALLBACK: &str = "Stream ended unexpectedly";
                                                        FALLBACK
                                                    });
                                                let chunk = serde_json::json!({
                                                    "type": "error",
                                                    "content": error_msg
                                                });
                                                callback(chunk.to_string());
                                                return Err(ClaudeError::ApiError {
                                                    provider: "deepseek".to_string(),
                                                    message: error_msg.to_string(),
                                                });
                                            }
                                        }

                                        let delta = &first_choice.delta;
                                        
                                        if let Some(content) = &delta.content {
                                            full_response.push_str(content);
                                            let chunk = serde_json::json!({
                                                "type": "content",
                                                "content": content
                                            });
                                            callback(chunk.to_string());
                                        }
                                        
                                        if let Some(reasoning) = &delta.reasoning_content {
                                            let chunk = serde_json::json!({
                                                "type": "thinking",
                                                "content": reasoning
                                            });
                                            callback(chunk.to_string());
                                        }
                                    }
                                }
                            } else if let Ok(error) = serde_json::from_str::<SSEError>(data) {
                                let chunk = serde_json::json!({
                                    "type": "error",
                                    "content": &error.error.message
                                });
                                callback(chunk.to_string());
                                return Err(ClaudeError::ApiError {
                                    provider: "deepseek".to_string(),
                                    message: error.error.message,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(ClaudeError::NetworkError {
                        message: e.to_string(),
                    });
                }
            }
        }

        Ok(full_response)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ClaudeError> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| ClaudeError::NetworkError {
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ClaudeError::ApiError {
                provider: "deepseek".to_string(),
                message: format!("HTTP {}", status),
            });
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<DeepSeekModel>,
        }

        #[derive(Deserialize)]
        struct DeepSeekModel {
            id: String,
            owned_by: Option<String>,
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| ClaudeError::SerializationError {
                message: e.to_string(),
            })?;

        let models = models_response
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id,
                name: None,
                owned_by: m.owned_by,
                context_length: None,
            })
            .collect();

        Ok(models)
    }

    async fn get_balance(&self) -> Result<BalanceInfo, ClaudeError> {
        let response = self
            .client
            .get(format!("{}/user/balance", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| ClaudeError::NetworkError {
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ClaudeError::ApiError {
                provider: "deepseek".to_string(),
                message: format!("HTTP {}", status),
            });
        }

        #[derive(Deserialize)]
        struct BalanceResponse {
            is_available: bool,
            balance_infos: Vec<BalanceData>,
        }

        #[derive(Deserialize)]
        struct BalanceData {
            currency: String,
            total_balance: String,
        }

        let balance_response: BalanceResponse = response
            .json()
            .await
            .map_err(|e| ClaudeError::SerializationError {
                message: e.to_string(),
            })?;

        let balance_info = balance_response.balance_infos.first();

        Ok(BalanceInfo {
            is_available: balance_response.is_available,
            total_balance: balance_info.map(|b| b.total_balance.clone()),
            currency: balance_info.map(|b| b.currency.clone()),
        })
    }

    fn box_clone(&self) -> Box<dyn Provider> {
        Box::new(self.clone())
    }
}

#[derive(Deserialize)]
struct SSEEvent {
    choices: Option<Vec<SSEDelta>>,
}

#[derive(Deserialize)]
struct SSEDelta {
    delta: DeltaContent,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeltaContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct SSEError {
    error: SSEErrorDetail,
}

#[derive(Deserialize)]
struct SSEErrorDetail {
    message: String,
    #[serde(default)]
    code: Option<String>,
}

use crate::api::providers::{Provider, ModelResponse, Usage, ModelInfo, BalanceInfo};
use crate::message::Message;
use crate::session::ClaudeError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SiliconFlowProvider {
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

impl SiliconFlowProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://api.siliconflow.cn/v1".to_string()),
            client: Client::new(),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                requests_remaining: 100,
                tokens_remaining: 100000,
                reset_at: std::time::Instant::now(),
            })),
        }
    }
}

impl Clone for SiliconFlowProvider {
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
    messages: Vec<ChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct SiliconFlowResponse {
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
    content: String,
}

#[derive(Deserialize)]
struct UsageResponse {
    prompt_tokens: u64,
    completion_tokens: u64,
    #[allow(dead_code)]
    total_tokens: u64,
}

#[async_trait]
impl Provider for SiliconFlowProvider {
    fn name(&self) -> &'static str {
        "siliconflow"
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
        let request = ChatCompletionRequest {
            model,
            messages: messages
                .iter()
                .map(|m| ChatMessage {
                    role: m.role.as_str(),
                    content: &m.content,
                })
                .collect(),
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
                provider: "siliconflow".to_string(),
                message: format!("HTTP {} - {}", status, body),
            });
        }

        let response: SiliconFlowResponse = response
            .json()
            .await
            .map_err(|e| ClaudeError::SerializationError {
                message: e.to_string(),
            })?;

        let content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ModelResponse {
            id: response.id,
            content,
            model: response.model,
            usage: Usage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
            stop_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }

    async fn stream_chat_completion(
        &self,
        model: &str,
        messages: &[&Message],
        max_tokens: usize,
        callback: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, ClaudeError> {
        let request = ChatCompletionRequest {
            model,
            messages: messages
                .iter()
                .map(|m| ChatMessage {
                    role: m.role.as_str(),
                    content: &m.content,
                })
                .collect(),
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
            return Err(ClaudeError::ApiError {
                provider: "siliconflow".to_string(),
                message: format!("HTTP {}", status),
            });
        }

        let mut full_response = String::new();

        use futures::StreamExt;

        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    if let Ok(line) = String::from_utf8(bytes.to_vec()) {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                break;
                            }
                            if let Ok(event) = serde_json::from_str::<SSEEvent>(data) {
                                if let Some(choices) = &event.choices {
                                    if let Some(first_choice) = choices.get(0) {
                                        if let Some(ref delta) = first_choice.delta.content {
                                            full_response.push_str(delta);
                                            callback(delta.clone());
                                        }
                                    }
                                }
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
                provider: "siliconflow".to_string(),
                message: format!("HTTP {}", status),
            });
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<SiliconFlowModel>,
        }

        #[derive(Deserialize)]
        struct SiliconFlowModel {
            id: String,
            name: Option<String>,
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
                name: m.name,
                owned_by: None,
                context_length: None,
            })
            .collect();

        Ok(models)
    }

    async fn get_balance(&self) -> Result<BalanceInfo, ClaudeError> {
        let response = self
            .client
            .get(format!("{}/user/info", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| ClaudeError::NetworkError {
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ClaudeError::ApiError {
                provider: "siliconflow".to_string(),
                message: format!("HTTP {}", status),
            });
        }

        #[derive(Deserialize)]
        struct BalanceResponse {
            data: BalanceData,
        }

        #[derive(Deserialize)]
        struct BalanceData {
            balance: String,
            charge_balance: Option<String>,
            total_balance: Option<String>,
        }

        let balance_response: BalanceResponse = response
            .json()
            .await
            .map_err(|e| ClaudeError::SerializationError {
                message: e.to_string(),
            })?;

        let total_balance = balance_response.data.total_balance
            .or(balance_response.data.charge_balance)
            .or(Some(balance_response.data.balance));

        Ok(BalanceInfo {
            is_available: true,
            total_balance,
            currency: Some("CNY".to_string()),
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
}

#[derive(Deserialize)]
struct DeltaContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

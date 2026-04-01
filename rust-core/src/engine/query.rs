use crate::message::Message;
use crate::session::{Session, ClaudeError};
use std::sync::Arc;

pub struct QueryEngine;

impl QueryEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_query(
        &self,
        session: &Arc<Session>,
        messages: &[Message],
    ) -> Result<String, ClaudeError> {
        let messages: Vec<_> = messages.iter().collect();

        let provider = session.provider.read();
        let provider = provider.as_ref().ok_or_else(|| ClaudeError::ConfigError {
            message: "No provider configured".to_string(),
        })?;

        let response = provider.chat_completion(
            &session.config.model,
            &messages,
            session.config.max_tokens,
        ).await?;

        Ok(response.content)
    }

    pub async fn execute_streaming_query(
        &self,
        session: &Arc<Session>,
        messages: &[Message],
        callback: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, ClaudeError> {
        let messages: Vec<_> = messages.iter().collect();

        let provider = session.provider.read();
        let provider = provider.as_ref().ok_or_else(|| ClaudeError::ConfigError {
            message: "No provider configured".to_string(),
        })?;

        let full_response = provider.stream_chat_completion(
            &session.config.model,
            &messages,
            session.config.max_tokens,
            callback,
        ).await?;

        Ok(full_response)
    }

    pub async fn execute_with_context(
        &self,
        session: &Arc<Session>,
        user_message: &str,
        system_prompt: Option<&str>,
    ) -> Result<String, ClaudeError> {
        let mut messages: Vec<Message> = Vec::new();

        if let Some(prompt) = system_prompt {
            messages.push(Message::system(prompt.to_string()));
        }

        let session_messages = session.messages.read();
        for msg in session_messages.iter() {
            messages.push(msg.clone());
        }
        drop(session_messages);

        messages.push(Message::user(user_message.to_string()));

        let messages_refs: Vec<&Message> = messages.iter().collect();
        let messages_slice: Vec<Message> = messages_refs.into_iter().cloned().collect();
        
        let provider = session.provider.read();
        let provider = provider.as_ref().ok_or_else(|| ClaudeError::ConfigError {
            message: "No provider configured".to_string(),
        })?;

        let messages_for_provider: Vec<&Message> = messages_slice.iter().collect();
        let response = provider.chat_completion(
            &session.config.model,
            &messages_for_provider,
            session.config.max_tokens,
        ).await?;

        Ok(response.content)
    }

    pub async fn execute_streaming_with_context(
        &self,
        session: &Arc<Session>,
        user_message: &str,
        system_prompt: Option<&str>,
        callback: &mut (dyn FnMut(String) + Send),
    ) -> Result<String, ClaudeError> {
        let mut messages: Vec<Message> = Vec::new();

        if let Some(prompt) = system_prompt {
            messages.push(Message::system(prompt.to_string()));
        }

        let session_messages = session.messages.read();
        for msg in session_messages.iter() {
            messages.push(msg.clone());
        }
        drop(session_messages);

        messages.push(Message::user(user_message.to_string()));

        let provider = session.provider.read();
        let provider = provider.as_ref().ok_or_else(|| ClaudeError::ConfigError {
            message: "No provider configured".to_string(),
        })?;

        let messages_for_provider: Vec<&Message> = messages.iter().collect();
        let full_response = provider.stream_chat_completion(
            &session.config.model,
            &messages_for_provider,
            session.config.max_tokens,
            callback,
        ).await?;

        Ok(full_response)
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn execute_query(
    session: &Arc<Session>,
    messages: &[Message],
) -> Result<String, ClaudeError> {
    let engine = QueryEngine::new();
    engine.execute_query(session, messages).await
}

pub async fn execute_streaming_query(
    session: &Arc<Session>,
    messages: &[Message],
    callback: &mut (dyn FnMut(String) + Send),
) -> Result<String, ClaudeError> {
    let engine = QueryEngine::new();
    engine.execute_streaming_query(session, messages, callback).await
}

use crate::message::{Message, Role};
use crate::session::{Session, ClaudeError};
use crate::db::{get_message_dao, MessageEntity};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

pub struct ConversationEngine {
    max_context_messages: usize,
}

impl ConversationEngine {
    pub fn new() -> Self {
        Self {
            max_context_messages: 100,
        }
    }

    pub fn with_max_context(mut self, max: usize) -> Self {
        self.max_context_messages = max;
        self
    }

    pub async fn submit_message(
        &self,
        session: &Arc<Session>,
        user_message: &str,
        system_prompt: Option<&str>,
    ) -> Result<ConversationResponse, ClaudeError> {
        let message_dao = get_message_dao().map_err(ClaudeError::DatabaseError)?;

        let user_msg_id = Uuid::new_v4().to_string();
        let parent_uuid = self.get_last_message_uuid(session)?;

        let user_entity = MessageEntity {
            id: user_msg_id.clone(),
            session_id: session.id.clone(),
            parent_uuid: parent_uuid.clone(),
            role: "user".to_string(),
            content: user_message.to_string(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            is_compact_summary: false,
            compact_boundary: None,
            order_index: message_dao.get_latest_order_index(&session.id)? + 1,
        };

        message_dao.insert(&user_entity)?;

        session.add_message(Message {
            role: Role::User,
            content: user_message.to_string(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            id: Some(user_msg_id),
        })?;

        let context_messages = self.build_context(session)?;
        
        let messages_for_api = self.build_api_messages(&context_messages, system_prompt)?;

        Ok(ConversationResponse {
            messages: messages_for_api,
            context_messages,
        })
    }

    pub async fn process_assistant_response(
        &self,
        session: &Arc<Session>,
        content: &str,
        thinking: Option<&str>,
    ) -> Result<String, ClaudeError> {
        let message_dao = get_message_dao().map_err(ClaudeError::DatabaseError)?;

        let assistant_msg_id = Uuid::new_v4().to_string();
        let parent_uuid = self.get_last_message_uuid(session)?;

        let assistant_entity = MessageEntity {
            id: assistant_msg_id.clone(),
            session_id: session.id.clone(),
            parent_uuid: parent_uuid,
            role: "assistant".to_string(),
            content: content.to_string(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: thinking.map(String::from),
            thinking_signature: None,
            timestamp: Utc::now(),
            is_compact_summary: false,
            compact_boundary: None,
            order_index: message_dao.get_latest_order_index(&session.id)? + 1,
        };

        message_dao.insert(&assistant_entity)?;

        session.add_message(Message {
            role: Role::Assistant,
            content: content.to_string(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: thinking.map(String::from),
            thinking_signature: None,
            timestamp: Utc::now(),
            id: Some(assistant_msg_id.clone()),
        })?;

        Ok(assistant_msg_id)
    }

    pub fn build_context(&self, session: &Arc<Session>) -> Result<Vec<Message>, ClaudeError> {
        let message_dao = get_message_dao()?;

        let compact_summaries = message_dao.get_compact_summaries(&session.id)?;
        
        let messages = if compact_summaries.is_empty() {
            let entities = message_dao.get_by_session_with_limit(&session.id, self.max_context_messages as i32)?;
            entities.into_iter().map(|m| self.entity_to_message(m)).collect()
        } else {
            let last_compact = compact_summaries.last().unwrap();
            let last_compact_index = last_compact.order_index;
            
            let mut context = Vec::new();
            
            for summary in compact_summaries {
                context.push(Message {
                    role: Role::System,
                    content: summary.content.clone(),
                    name: None,
                    tool_call_id: None,
                    tool_name: None,
                    tool_input: None,
                    thinking: None,
                    thinking_signature: None,
                    timestamp: summary.timestamp,
                    id: Some(summary.id),
                });
            }

            // Only fetch recent messages after the last compact boundary, with limit
            let recent_messages = message_dao.get_by_session(&session.id)?
                .into_iter()
                .filter(|m| m.order_index > last_compact_index)
                .collect::<Vec<_>>();

            // Apply max_context_messages limit to recent messages
            let start = recent_messages.len().saturating_sub(self.max_context_messages);
            for msg in recent_messages.into_iter().skip(start) {
                context.push(self.entity_to_message(msg));
            }

            context
        };

        Ok(messages)
    }

    fn entity_to_message(&self, entity: MessageEntity) -> Message {
        Message {
            role: match entity.role.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "system" => Role::System,
                "tool_use" => Role::ToolUse,
                "tool_result" => Role::ToolResult,
                _ => Role::User,
            },
            content: entity.content,
            name: entity.name,
            tool_call_id: entity.tool_call_id,
            tool_name: entity.tool_name,
            tool_input: entity.tool_input.and_then(|s| serde_json::from_str(&*s).ok()),
            thinking: entity.thinking,
            thinking_signature: entity.thinking_signature,
            timestamp: entity.timestamp,
            id: Some(entity.id),
        }
    }

    pub fn build_api_messages(
        &self,
        context: &[Message],
        system_prompt: Option<&str>,
    ) -> Result<Vec<Message>, ClaudeError> {
        let mut messages = Vec::new();

        if let Some(prompt) = system_prompt {
            messages.push(Message::system(prompt));
        }

        for msg in context {
            messages.push(msg.clone());
        }

        Ok(messages)
    }

    pub fn compact_context(
        &self,
        session: &Arc<Session>,
        summary: &str,
        boundary_message_id: &str,
    ) -> Result<(), ClaudeError> {
        let message_dao = get_message_dao().map_err(ClaudeError::DatabaseError)?;

        let compact_msg_id = Uuid::new_v4().to_string();
        let parent_uuid = self.get_last_message_uuid(session)?;

        let compact_entity = MessageEntity {
            id: compact_msg_id,
            session_id: session.id.clone(),
            parent_uuid,
            role: "system".to_string(),
            content: summary.to_string(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            is_compact_summary: true,
            compact_boundary: Some(boundary_message_id.to_string()),
            order_index: message_dao.get_latest_order_index(&session.id)? + 1,
        };

        message_dao.insert(&compact_entity)?;

        Ok(())
    }

    pub fn snip_messages(
        &self,
        session: &Arc<Session>,
        before_index: i32,
    ) -> Result<i32, ClaudeError> {
        let message_dao = get_message_dao().map_err(ClaudeError::DatabaseError)?;

        let count = message_dao.delete_before_index(&session.id, before_index)?;
        Ok(count)
    }

    fn get_last_message_uuid(&self, session: &Arc<Session>) -> Result<Option<String>, ClaudeError> {
        let messages = session.messages.read();
        Ok(messages.last().and_then(|m| m.id.clone()))
    }

    pub async fn get_conversation_history(
        &self,
        session_id: &str,
    ) -> Result<Vec<Message>, ClaudeError> {
        let message_dao = get_message_dao().map_err(ClaudeError::DatabaseError)?;

        let entities = message_dao.get_by_session(session_id)?;
        
        let messages = entities.into_iter().map(|e| Message {
            role: match e.role.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "system" => Role::System,
                "tool_use" => Role::ToolUse,
                "tool_result" => Role::ToolResult,
                _ => Role::User,
            },
            content: e.content,
            name: e.name,
            tool_call_id: e.tool_call_id,
            tool_name: e.tool_name,
            tool_input: e.tool_input.and_then(|s| serde_json::from_str(&s).ok()),
            thinking: e.thinking,
            thinking_signature: e.thinking_signature,
            timestamp: e.timestamp,
            id: Some(e.id),
        }).collect();

        Ok(messages)
    }
}

impl Default for ConversationEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConversationResponse {
    pub messages: Vec<Message>,
    pub context_messages: Vec<Message>,
}

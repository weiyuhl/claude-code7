use crate::message::Message;
use crate::session::{Session, ClaudeError};
use crate::engine::conversation::ConversationEngine;
use std::sync::Arc;

pub struct ContextManager {
    conversation_engine: ConversationEngine,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            conversation_engine: ConversationEngine::new(),
        }
    }

    pub fn with_conversation_engine(mut self, engine: ConversationEngine) -> Self {
        self.conversation_engine = engine;
        self
    }

    pub fn auto_compact(
        &self,
        session: &Arc<Session>,
        token_threshold: usize,
    ) -> Result<bool, ClaudeError> {
        let context = self.conversation_engine.build_context(session)?;
        
        let total_tokens = self.estimate_tokens(&context);
        
        if total_tokens < token_threshold {
            return Ok(false);
        }

        let compact_boundary = self.find_compact_boundary(&context, token_threshold)?;
        
        let summary = self.generate_compact_summary(&context[..compact_boundary])?;
        
        let boundary_msg_id = context[compact_boundary].id.clone().unwrap_or_default();
        
        self.conversation_engine.compact_context(session, &summary, &boundary_msg_id)?;
        
        self.conversation_engine.snip_messages(session, compact_boundary as i32)?;

        Ok(true)
    }

    pub async fn manual_compact(
        &self,
        session: &Arc<Session>,
        summary: &str,
        up_to_message_id: &str,
    ) -> Result<(), ClaudeError> {
        self.conversation_engine.compact_context(session, summary, up_to_message_id)?;
        Ok(())
    }

    pub fn snip_before(
        &self,
        session: &Arc<Session>,
        message_index: i32,
    ) -> Result<i32, ClaudeError> {
        self.conversation_engine.snip_messages(session, message_index)
    }

    pub fn get_context_window(
        &self,
        session: &Arc<Session>,
        max_messages: Option<usize>,
    ) -> Result<Vec<Message>, ClaudeError> {
        let context = self.conversation_engine.build_context(session)?;
        
        if let Some(max) = max_messages {
            Ok(context.into_iter().rev().take(max).rev().collect())
        } else {
            Ok(context)
        }
    }

    pub fn build_system_prompt(
        &self,
        session: &Arc<Session>,
        custom_prompt: Option<&str>,
    ) -> Result<String, ClaudeError> {
        let mut prompt = String::new();

        prompt.push_str("You are Claude, an AI assistant powered by Anthropic's Claude models.\n\n");

        if let Some(custom) = custom_prompt {
            prompt.push_str(custom);
            prompt.push_str("\n\n");
        }

        let context = self.conversation_engine.build_context(session)?;
        let token_count = self.estimate_tokens(&context);
        
        prompt.push_str(&format!("Current conversation context contains {} tokens.\n", token_count));
        prompt.push_str("Be helpful, harmless, and honest in all interactions.\n");

        Ok(prompt)
    }

    pub fn estimate_tokens(&self, messages: &[Message]) -> usize {
        let mut total = 0;
        for msg in messages {
            total += msg.content.len() / 4;
            if let Some(thinking) = &msg.thinking {
                total += thinking.len() / 4;
            }
        }
        total
    }

    fn find_compact_boundary(
        &self,
        messages: &[Message],
        threshold: usize,
    ) -> Result<usize, ClaudeError> {
        let mut tokens = 0;
        let mut boundary = 0;
        let target = threshold / 2;

        for (i, msg) in messages.iter().enumerate() {
            tokens += msg.content.len() / 4;
            if tokens >= target && msg.role == crate::message::Role::Assistant {
                boundary = i + 1;
                break;
            }
        }

        if boundary == 0 {
            boundary = messages.len() / 4;
        }

        Ok(boundary)
    }

    fn generate_compact_summary(&self, messages: &[Message]) -> Result<String, ClaudeError> {
        if messages.is_empty() {
            return Ok("No previous context.".to_string());
        }

        let mut summary = String::from("[Conversation Summary]\n");
        
        let mut user_messages = 0;
        let mut assistant_messages = 0;
        let mut tool_uses = 0;

        for msg in messages {
            match msg.role {
                crate::message::Role::User => user_messages += 1,
                crate::message::Role::Assistant => assistant_messages += 1,
                crate::message::Role::ToolUse => tool_uses += 1,
                _ => {}
            }
        }

        summary.push_str(&format!(
            "- {} user messages, {} assistant responses, {} tool uses\n",
            user_messages, assistant_messages, tool_uses
        ));

        if let Some(first) = messages.first() {
            let first_content = first.content.chars().take(200).collect::<String>();
            summary.push_str(&format!("- First topic: {}\n", first_content));
        }

        if let Some(last) = messages.last() {
            let last_content = last.content.chars().take(200).collect::<String>();
            summary.push_str(&format!("- Last topic: {}\n", last_content));
        }

        summary.push_str("[End Summary - Continue from here]\n");

        Ok(summary)
    }

    pub fn get_conversation_stats(
        &self,
        session: &Arc<Session>,
    ) -> Result<ConversationStats, ClaudeError> {
        let context = self.conversation_engine.build_context(session)?;
        
        let mut user_messages = 0;
        let mut assistant_messages = 0;
        let mut system_messages = 0;
        let mut tool_uses = 0;
        let mut tool_results = 0;
        let mut total_tokens = 0;

        for msg in &context {
            total_tokens += msg.content.len() / 4;
            
            match msg.role {
                crate::message::Role::User => user_messages += 1,
                crate::message::Role::Assistant => assistant_messages += 1,
                crate::message::Role::System => system_messages += 1,
                crate::message::Role::ToolUse => tool_uses += 1,
                crate::message::Role::ToolResult => tool_results += 1,
            }
        }

        Ok(ConversationStats {
            total_messages: context.len(),
            user_messages,
            assistant_messages,
            system_messages,
            tool_uses,
            tool_results,
            total_tokens,
            compact_summaries: 0,
        })
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConversationStats {
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub system_messages: usize,
    pub tool_uses: usize,
    pub tool_results: usize,
    pub total_tokens: usize,
    pub compact_summaries: usize,
}

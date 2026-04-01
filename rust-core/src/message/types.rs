use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    ToolUse,
    ToolResult,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::ToolUse => "tool_use",
            Role::ToolResult => "tool_result",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_signature: Option<String>,
    #[serde(default)]
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            id: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            id: None,
        }
    }

    pub fn tool_use(tool_name: impl Into<String>, tool_call_id: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            role: Role::ToolUse,
            content: String::new(),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
            tool_name: Some(tool_name.into()),
            tool_input: Some(input),
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            id: None,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::ToolResult,
            content: content.into(),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
            tool_name: None,
            tool_input: None,
            thinking: None,
            thinking_signature: None,
            timestamp: Utc::now(),
            id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageList {
    pub messages: Vec<Message>,
}

impl MessageList {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity),
        }
    }
}

impl Default for MessageList {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for MessageList {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Message>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

impl From<Vec<Message>> for MessageList {
    fn from(messages: Vec<Message>) -> Self {
        Self { messages }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = Message::assistant("Hi there");
        assert_eq!(assistant_msg.role, Role::Assistant);

        let system_msg = Message::system("You are helpful");
        assert_eq!(system_msg.role, Role::System);
    }

    #[test]
    fn test_tool_messages() {
        let tool_use = Message::tool_use("file_read", "call_123", serde_json::json!({"path": "/test.txt"}));
        assert_eq!(tool_use.role, Role::ToolUse);
        assert_eq!(tool_use.tool_name.as_ref().unwrap(), "file_read");

        let tool_result = Message::tool_result("call_123", "File contents here");
        assert_eq!(tool_result.role, Role::ToolResult);
        assert_eq!(tool_result.content, "File contents here");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("Test message");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content, msg.content);
    }
}

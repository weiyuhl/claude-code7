use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("API error: {provider} - {message}")]
    ApiError { provider: String, message: String },

    #[error("Tool error: {tool} - {message}")]
    ToolError { tool: String, message: String },

    #[error("Session error: {session_id} - {message}")]
    SessionError { session_id: String, message: String },

    #[error("Permission error: {resource} - {action}")]
    PermissionError { resource: String, action: String },

    #[error("IO error: {path} - {message}")]
    IoError { path: String, message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Rate limit error: {provider} - retry after {retry_after}s")]
    RateLimitError { provider: String, retry_after: u64 },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl serde::Serialize for ClaudeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            error: String,
            error_type: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            provider: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tool: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            session_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            resource: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            path: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            retry_after: Option<u64>,
        }

        let (error_type, provider, tool, session_id, resource, path, retry_after) = match self {
            ClaudeError::ApiError { provider, message: _ } => {
                ("api_error".to_string(), Some(provider.clone()), None, None, None, None, None)
            }
            ClaudeError::ToolError { tool, message: _ } => {
                ("tool_error".to_string(), None, Some(tool.clone()), None, None, None, None)
            }
            ClaudeError::SessionError { session_id, message: _ } => {
                ("session_error".to_string(), None, None, Some(session_id.clone()), None, None, None)
            }
            ClaudeError::PermissionError { resource, action: _ } => {
                ("permission_error".to_string(), None, None, None, Some(resource.clone()), None, None)
            }
            ClaudeError::IoError { path, message: _ } => {
                ("io_error".to_string(), None, None, None, None, Some(path.clone()), None)
            }
            ClaudeError::ConfigError { message: _ } => ("config_error".to_string(), None, None, None, None, None, None),
            ClaudeError::SerializationError { message: _ } => ("serialization_error".to_string(), None, None, None, None, None, None),
            ClaudeError::NetworkError { message: _ } => ("network_error".to_string(), None, None, None, None, None, None),
            ClaudeError::RateLimitError { provider, retry_after } => ("rate_limit_error".to_string(), Some(provider.clone()), None, None, None, None, Some(*retry_after)),
            ClaudeError::AuthError { message: _ } => ("auth_error".to_string(), None, None, None, None, None, None),
            ClaudeError::Unknown(_) => ("unknown_error".to_string(), None, None, None, None, None, None),
        };

        ErrorResponse {
            error: self.to_string(),
            error_type,
            provider,
            tool,
            session_id,
            resource,
            path,
            retry_after,
        }.serialize(serializer)
    }
}

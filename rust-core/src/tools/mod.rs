pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn execute(&self, input: serde_json::Value, context: &ToolContext) -> Result<ToolResult, ToolError>;
}

pub struct ToolContext {
    pub working_dir: String,
    pub session_id: String,
}

pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
}

pub struct ToolError {
    pub message: String,
}

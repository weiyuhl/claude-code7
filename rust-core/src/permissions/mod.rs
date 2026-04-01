pub struct PermissionContext {
    pub mode: PermissionMode,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PermissionMode {
    Default,
    AcceptEdits,
    BypassPermissions,
    DontAsk,
    Plan,
}

impl Default for PermissionContext {
    fn default() -> Self {
        Self {
            mode: PermissionMode::Default,
            allowed_tools: Vec::new(),
            disallowed_tools: Vec::new(),
            working_directory: None,
        }
    }
}

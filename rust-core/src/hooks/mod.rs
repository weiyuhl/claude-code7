pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn on_event(&self, event: &HookEvent) -> Result<(), HookError>;
}

pub enum HookEvent {
    Setup,
    SessionStart,
    PreToolUse,
    PostToolUse,
    Notification,
}

pub struct HookError {
    pub message: String,
}

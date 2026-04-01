pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &[&str]) -> Result<String, CommandError>;
}

pub struct CommandError {
    pub message: String,
}

pub struct AppState {
    pub messages: Vec<crate::message::Message>,
    pub session_id: Option<String>,
    pub config: crate::session::SessionConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            session_id: None,
            config: crate::session::SessionConfig::default(),
        }
    }
}

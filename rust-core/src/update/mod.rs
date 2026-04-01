pub struct AutoUpdater {
    pub enabled: bool,
}

impl AutoUpdater {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

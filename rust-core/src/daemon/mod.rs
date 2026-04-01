pub struct Daemon {
    pub running: bool,
}

impl Daemon {
    pub fn new() -> Self {
        Self { running: false }
    }
}

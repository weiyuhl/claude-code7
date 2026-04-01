pub struct Sandbox {
    pub enabled: bool,
}

impl Sandbox {
    pub fn new() -> Self {
        Self { enabled: false }
    }
}

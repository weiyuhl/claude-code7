pub struct BuddyCoordinator {
    pub agents: Vec<String>,
}

impl BuddyCoordinator {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
        }
    }
}

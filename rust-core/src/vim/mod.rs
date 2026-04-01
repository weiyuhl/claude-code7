#[derive(Debug, Clone, Copy)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

pub struct VimState {
    pub mode: VimMode,
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
        }
    }
}

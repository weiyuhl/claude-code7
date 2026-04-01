mod c_api;

pub use c_api::*;

pub struct Bridge {
    pub mode: BridgeMode,
}

#[derive(Debug)]
pub enum BridgeMode {
    Ssh,
    Direct,
    Cloud,
}

impl Bridge {
    pub fn new() -> Self {
        Self {
            mode: BridgeMode::Direct,
        }
    }
}

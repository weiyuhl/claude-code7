pub struct Voice {
    pub stt_enabled: bool,
    pub tts_enabled: bool,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            stt_enabled: false,
            tts_enabled: false,
        }
    }
}

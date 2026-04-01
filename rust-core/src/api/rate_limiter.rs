pub struct RateLimiter {
    requests_per_minute: u32,
    tokens_per_minute: u32,
    current_requests: u32,
    current_tokens: u32,
    window_start: std::time::Instant,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32, tokens_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            tokens_per_minute,
            current_requests: 0,
            current_tokens: 0,
            window_start: std::time::Instant::now(),
        }
    }

    pub fn check(&mut self, tokens_needed: u32) -> bool {
        self.reset_if_needed();

        self.current_requests < self.requests_per_minute
            && self.current_tokens + tokens_needed <= self.tokens_per_minute
    }

    pub fn consume(&mut self, tokens: u32) {
        self.reset_if_needed();
        self.current_requests += 1;
        self.current_tokens += tokens;
    }

    pub fn retry_after(&self) -> u64 {
        let elapsed = self.window_start.elapsed();
        if elapsed < std::time::Duration::from_secs(60) {
            60 - elapsed.as_secs()
        } else {
            0
        }
    }

    fn reset_if_needed(&mut self) {
        if self.window_start.elapsed() >= std::time::Duration::from_secs(60) {
            self.current_requests = 0;
            self.current_tokens = 0;
            self.window_start = std::time::Instant::now();
        }
    }
}

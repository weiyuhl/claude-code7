pub struct CostTracker {
    pub total_cost: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            total_cost: 0.0,
            input_tokens: 0,
            output_tokens: 0,
        }
    }

    pub fn add_usage(&mut self, input: u64, output: u64, cost_per_token: f64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.total_cost += (input + output) as f64 * cost_per_token;
    }
}

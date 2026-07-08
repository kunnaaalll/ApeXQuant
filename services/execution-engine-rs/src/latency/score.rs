#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyScore {
    pub score: u8, // 0-100
}

impl LatencyScore {
    pub fn calculate(total_latency_ms: u64) -> Result<Self, &'static str> {
        let penalty = if total_latency_ms > 200 {
            100
        } else {
            (total_latency_ms / 2) as u8
        };

        let score = 100u8.saturating_sub(penalty);
        Ok(Self { score })
    }
}

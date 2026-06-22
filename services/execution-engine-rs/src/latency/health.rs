#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyState {
    Excellent,
    Healthy,
    Warning,
    Critical,
}

impl LatencyState {
    pub fn evaluate(total_latency_ms: u64) -> Result<Self, &'static str> {
        if total_latency_ms <= 10 {
            Ok(LatencyState::Excellent)
        } else if total_latency_ms <= 50 {
            Ok(LatencyState::Healthy)
        } else if total_latency_ms <= 150 {
            Ok(LatencyState::Warning)
        } else {
            Ok(LatencyState::Critical)
        }
    }
}

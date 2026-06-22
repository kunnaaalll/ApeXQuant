#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResiliencyState {
    Fast,
    Normal,
    Slow,
    Broken,
}

impl ResiliencyState {
    pub fn evaluate(recovery_ms: u64) -> Result<Self, &'static str> {
        if recovery_ms <= 10 {
            Ok(ResiliencyState::Fast)
        } else if recovery_ms <= 50 {
            Ok(ResiliencyState::Normal)
        } else if recovery_ms <= 200 {
            Ok(ResiliencyState::Slow)
        } else {
            Ok(ResiliencyState::Broken)
        }
    }
}

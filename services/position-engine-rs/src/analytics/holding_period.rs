use std::time::Duration;

pub struct HoldingPeriodAnalyzer;

impl HoldingPeriodAnalyzer {
    pub fn calculate_efficiency(realized_pnl: f32, holding_duration: Duration) -> f32 {
        let hours = holding_duration.as_secs_f32() / 3600.0;
        if hours > 0.0 {
            realized_pnl / hours
        } else {
            0.0
        }
    }
}

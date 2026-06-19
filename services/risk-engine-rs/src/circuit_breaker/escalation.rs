#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskEscalationLevel {
    Info = 0,
    Watch = 1,
    Warning = 2,
    Critical = 3,
    Emergency = 4,
}

#[derive(Debug, Clone)]
pub struct RiskEscalationAssessment {
    pub level: RiskEscalationLevel,
}

impl RiskEscalationAssessment {
    pub fn evaluate(
        drawdown_score: u8,
        var_score: u8,
        tail_risk_score: u8,
        liquidity_score: u8,
        exposure_score: u8,
        correlation_score: u8,
        black_swan_score: u8,
    ) -> Self {
        // Aggregate the scores (assume scores are 0-10 where 10 is worst)
        let total = drawdown_score as u32
            + var_score as u32
            + tail_risk_score as u32
            + liquidity_score as u32
            + exposure_score as u32
            + correlation_score as u32
            + black_swan_score as u32;

        let level = if black_swan_score >= 8 || tail_risk_score >= 9 || total >= 50 {
            RiskEscalationLevel::Emergency
        } else if total >= 35 {
            RiskEscalationLevel::Critical
        } else if total >= 20 {
            RiskEscalationLevel::Warning
        } else if total >= 10 {
            RiskEscalationLevel::Watch
        } else {
            RiskEscalationLevel::Info
        };

        Self { level }
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityState {
    Normal = 0,
    Elevated = 1,
    Danger = 2,
    Critical = 3,
    Frozen = 4,
}

#[derive(Debug, Clone)]
pub struct RiskSeverityAssessment {
    pub overall_risk_restriction_score: Decimal,
    pub state: SeverityState,
}

impl RiskSeverityAssessment {
    pub fn new(mut score: Decimal) -> Self {
        // Bound: 0 <= score <= 100
        if score.is_sign_negative() {
            score = Decimal::ZERO;
        }
        let max_score = Decimal::new(100, 0);
        if score > max_score {
            score = max_score;
        }

        let mut assessment = Self {
            overall_risk_restriction_score: score,
            state: SeverityState::Normal,
        };
        assessment.update_state();
        assessment
    }

    fn update_state(&mut self) {
        self.state = if self.overall_risk_restriction_score >= Decimal::new(90, 0) {
            SeverityState::Frozen
        } else if self.overall_risk_restriction_score >= Decimal::new(75, 0) {
            SeverityState::Critical
        } else if self.overall_risk_restriction_score >= Decimal::new(50, 0) {
            SeverityState::Danger
        } else if self.overall_risk_restriction_score >= Decimal::new(25, 0) {
            SeverityState::Elevated
        } else {
            SeverityState::Normal
        };
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLimitState {
    Healthy = 0,
    Elevated = 1,
    Warning = 2,
    Danger = 3,
    Frozen = 4,
}

#[derive(Debug, Clone)]
pub struct RiskLimitAssessment {
    pub max_portfolio_risk: Decimal,
    pub max_position_risk: Decimal,
    pub max_daily_loss: Decimal,
    pub max_weekly_loss: Decimal,
    pub max_monthly_loss: Decimal,
    pub remaining_capacity: Decimal,
    pub state: RiskLimitState,
}

impl RiskLimitAssessment {
    pub fn new(
        max_portfolio_risk: Decimal,
        max_position_risk: Decimal,
        max_daily_loss: Decimal,
        max_weekly_loss: Decimal,
        max_monthly_loss: Decimal,
        remaining_capacity: Decimal,
    ) -> Self {
        let mut assessment = Self {
            max_portfolio_risk,
            max_position_risk,
            max_daily_loss,
            max_weekly_loss,
            max_monthly_loss,
            remaining_capacity,
            state: RiskLimitState::Healthy,
        };
        assessment.update_state();
        assessment
    }

    pub fn update_capacity(&mut self, new_capacity: Decimal) {
        self.remaining_capacity = if new_capacity.is_sign_negative() {
            Decimal::ZERO
        } else {
            new_capacity
        };
        self.update_state();
    }

    fn update_state(&mut self) {
        // Evaluate remaining capacity as a percentage of max_portfolio_risk
        // to determine the state. If max_portfolio_risk is zero, state is Frozen.
        if self.max_portfolio_risk.is_zero() || self.remaining_capacity.is_zero() {
            self.state = RiskLimitState::Frozen;
            return;
        }

        let ratio = self.remaining_capacity / self.max_portfolio_risk;

        self.state = if ratio >= Decimal::new(75, 2) {
            RiskLimitState::Healthy // >= 75%
        } else if ratio >= Decimal::new(50, 2) {
            RiskLimitState::Elevated // >= 50%
        } else if ratio >= Decimal::new(25, 2) {
            RiskLimitState::Warning // >= 25%
        } else {
            RiskLimitState::Danger // < 25%
        };
    }
}

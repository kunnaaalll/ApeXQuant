use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawdownState {
    Safe = 0,
    Caution = 1,
    Danger = 2,
    Critical = 3,
    Frozen = 4,
}

#[derive(Debug, Clone)]
pub struct DrawdownLimitAssessment {
    pub current_drawdown: Decimal,
    pub max_drawdown_limit: Decimal,
    pub remaining_drawdown_capacity: Decimal,
    pub state: DrawdownState,
}

impl DrawdownLimitAssessment {
    pub fn new(current_drawdown: Decimal, max_drawdown_limit: Decimal) -> Self {
        let mut remaining = max_drawdown_limit - current_drawdown;
        
        // Guarantee: remaining_drawdown_capacity >= 0
        if remaining.is_sign_negative() {
            remaining = Decimal::ZERO;
        }

        let mut assessment = Self {
            current_drawdown,
            max_drawdown_limit,
            remaining_drawdown_capacity: remaining,
            state: DrawdownState::Safe,
        };
        assessment.update_state();
        assessment
    }

    pub fn update_drawdown(&mut self, current_drawdown: Decimal) {
        self.current_drawdown = current_drawdown;
        
        let mut remaining = self.max_drawdown_limit - self.current_drawdown;
        if remaining.is_sign_negative() {
            remaining = Decimal::ZERO;
        }
        
        self.remaining_drawdown_capacity = remaining;
        self.update_state();
    }

    fn update_state(&mut self) {
        if self.max_drawdown_limit.is_zero() || self.remaining_drawdown_capacity.is_zero() {
            self.state = DrawdownState::Frozen;
            return;
        }

        let ratio = self.remaining_drawdown_capacity / self.max_drawdown_limit;

        self.state = if ratio >= Decimal::new(60, 2) {
            DrawdownState::Safe
        } else if ratio >= Decimal::new(30, 2) {
            DrawdownState::Caution
        } else if ratio >= Decimal::new(10, 2) {
            DrawdownState::Danger
        } else {
            DrawdownState::Critical
        };
    }
}

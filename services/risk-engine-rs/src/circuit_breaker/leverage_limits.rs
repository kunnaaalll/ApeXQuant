use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeverageState {
    Normal = 0,
    Elevated = 1,
    Danger = 2,
    Critical = 3,
    Collapse = 4,
}

#[derive(Debug, Clone)]
pub struct LeverageAssessment {
    pub gross_leverage: Decimal,
    pub effective_leverage: Decimal,
    pub hidden_leverage: Decimal,
    pub state: LeverageState,
}

impl LeverageAssessment {
    pub fn new(
        gross_leverage: Decimal,
        effective_leverage: Decimal,
        hidden_leverage: Decimal,
    ) -> Self {
        // Guarantee no negative leverage
        let gross = if gross_leverage.is_sign_negative() {
            Decimal::ZERO
        } else {
            gross_leverage
        };
        let effective = if effective_leverage.is_sign_negative() {
            Decimal::ZERO
        } else {
            effective_leverage
        };
        let hidden = if hidden_leverage.is_sign_negative() {
            Decimal::ZERO
        } else {
            hidden_leverage
        };

        let mut assessment = Self {
            gross_leverage: gross,
            effective_leverage: effective,
            hidden_leverage: hidden,
            state: LeverageState::Normal,
        };
        assessment.update_state();
        assessment
    }

    fn update_state(&mut self) {
        let total_leverage = self.gross_leverage + self.hidden_leverage;

        // Assuming thresholds for leverage limits:
        // > 5.0x = Collapse
        // > 4.0x = Critical
        // > 3.0x = Danger
        // > 2.0x = Elevated
        // <= 2.0x = Normal

        self.state = if total_leverage > Decimal::new(50, 1) {
            // 5.0
            LeverageState::Collapse
        } else if total_leverage > Decimal::new(40, 1) {
            // 4.0
            LeverageState::Critical
        } else if total_leverage > Decimal::new(30, 1) {
            // 3.0
            LeverageState::Danger
        } else if total_leverage > Decimal::new(20, 1) {
            // 2.0
            LeverageState::Elevated
        } else {
            LeverageState::Normal
        };
    }
}

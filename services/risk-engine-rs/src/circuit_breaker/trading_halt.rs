#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TradingHaltState {
    Open = 0,
    SlowMode = 1,
    Restricted = 2,
    Blocked = 3,
    EmergencyStop = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HaltTriggers {
    pub drawdown_breach: bool,
    pub tail_risk_event: bool,
    pub liquidity_collapse: bool,
    pub volatility_explosion: bool,
    pub black_swan_detected: bool,
}

impl HaltTriggers {
    pub fn no_triggers() -> Self {
        Self {
            drawdown_breach: false,
            tail_risk_event: false,
            liquidity_collapse: false,
            volatility_explosion: false,
            black_swan_detected: false,
        }
    }

    pub fn any_triggered(&self) -> bool {
        self.drawdown_breach
            || self.tail_risk_event
            || self.liquidity_collapse
            || self.volatility_explosion
            || self.black_swan_detected
    }

    pub fn evaluate_state(&self) -> TradingHaltState {
        let mut severity = 0;

        if self.drawdown_breach { severity += 2; }
        if self.tail_risk_event { severity += 2; }
        if self.liquidity_collapse { severity += 1; }
        if self.volatility_explosion { severity += 1; }
        if self.black_swan_detected { severity += 4; }

        match severity {
            0 => TradingHaltState::Open,
            1 => TradingHaltState::SlowMode,
            2 => TradingHaltState::Restricted,
            3 => TradingHaltState::Blocked,
            _ => TradingHaltState::EmergencyStop,
        }
    }
}

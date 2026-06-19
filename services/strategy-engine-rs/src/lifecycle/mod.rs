use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Born,
    Growing,
    Mature,
    Declining,
    Dying,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleProfile {
    pub state: LifecycleState,
    pub age_cycles: u64,
    pub trade_count: u64,
    pub performance_trend: Decimal,
    pub confidence_trend: Decimal,
}

impl LifecycleProfile {
    pub fn new() -> Self {
        Self {
            state: LifecycleState::Born,
            age_cycles: 0,
            trade_count: 0,
            performance_trend: Decimal::from(0),
            confidence_trend: Decimal::from(0),
        }
    }
}

impl Default for LifecycleProfile {
    fn default() -> Self {
        Self::new()
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelligenceSnapshot {
    pub expectancy: Decimal,
    pub stability: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfidenceSnapshot {
    pub score: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreakSnapshot {
    pub win_streak: u32,
    pub loss_streak: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftSnapshot {
    pub edge_drift: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySnapshot {
    pub items_count: usize,
}

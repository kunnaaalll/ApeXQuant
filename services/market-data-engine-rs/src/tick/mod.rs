// Tick domain

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub spread: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickQuality {
    Excellent,
    Good,
    Normal,
    Poor,
    Invalid,
}

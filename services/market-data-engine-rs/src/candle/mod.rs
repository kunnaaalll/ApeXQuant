// Candle domain

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OHLCV {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandleQuality {
    Elite,
    Strong,
    Normal,
    Weak,
    Corrupted,
}

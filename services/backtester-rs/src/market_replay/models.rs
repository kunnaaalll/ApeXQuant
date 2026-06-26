use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub symbol: String,
    pub timestamp: OffsetDateTime,
    pub bid: Decimal,
    pub ask: Decimal,
    pub bid_size: Decimal,
    pub ask_size: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candle {
    pub symbol: String,
    pub timestamp: OffsetDateTime,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayEvent {
    Tick(Tick),
    Candle(Candle),
}

impl ReplayEvent {
    pub fn timestamp(&self) -> OffsetDateTime {
        match self {
            ReplayEvent::Tick(t) => t.timestamp,
            ReplayEvent::Candle(c) => c.timestamp,
        }
    }
}

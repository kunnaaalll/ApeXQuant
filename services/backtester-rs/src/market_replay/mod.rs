//! Market Replay Module
//!
//! Tick, candle, multi-symbol, multi-timeframe replay engine.

pub enum ReplaySpeed {
    OneX,
    TenX,
    OneHundredX,
    OneThousandX,
    Unlimited,
}

pub struct MarketReplayEngine {
    pub speed: ReplaySpeed,
}

use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThroughputGrade {
    Excellent,
    Normal,
    Slow,
    Critical,
}

pub struct ThroughputMetrics {
    pub ticks_per_sec: Decimal,
    pub candles_per_sec: Decimal,
    pub events_per_sec: Decimal,
    pub grade: ThroughputGrade,
}

pub struct ThroughputEngine {
    window_start: DateTime<Utc>,
    ticks_in_window: u64,
    candles_in_window: u64,
    events_in_window: u64,
}

impl Default for ThroughputEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ThroughputEngine {
    pub fn new() -> Self {
        Self {
            window_start: Utc::now(),
            ticks_in_window: 0,
            candles_in_window: 0,
            events_in_window: 0,
        }
    }

    pub fn record_tick(&mut self) {
        self.ticks_in_window = self.ticks_in_window.saturating_add(1);
    }

    pub fn record_candle(&mut self) {
        self.candles_in_window = self.candles_in_window.saturating_add(1);
    }

    pub fn record_event(&mut self) {
        self.events_in_window = self.events_in_window.saturating_add(1);
    }

    pub fn calculate_metrics(&mut self, now: DateTime<Utc>) -> Option<ThroughputMetrics> {
        let duration = now.signed_duration_since(self.window_start).num_milliseconds();
        if duration < 1000 {
            return None; // Wait for at least a second
        }

        let duration_dec = Decimal::from(duration) / Decimal::from(1000);
        
        let ticks_per_sec = Decimal::from(self.ticks_in_window) / duration_dec;
        let candles_per_sec = Decimal::from(self.candles_in_window) / duration_dec;
        let events_per_sec = Decimal::from(self.events_in_window) / duration_dec;

        let grade = if ticks_per_sec > Decimal::from(10_000) {
            ThroughputGrade::Excellent
        } else if ticks_per_sec > Decimal::from(1_000) {
            ThroughputGrade::Normal
        } else if ticks_per_sec > Decimal::from(100) {
            ThroughputGrade::Slow
        } else {
            ThroughputGrade::Critical
        };

        // Reset window
        self.window_start = now;
        self.ticks_in_window = 0;
        self.candles_in_window = 0;
        self.events_in_window = 0;

        Some(ThroughputMetrics {
            ticks_per_sec,
            candles_per_sec,
            events_per_sec,
            grade,
        })
    }
}

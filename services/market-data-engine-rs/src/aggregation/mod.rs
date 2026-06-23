// Aggregation Engine

use crate::tick::Tick;
use crate::candle::{OHLCV, Timeframe};
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;

pub struct CandleAggregator {
    timeframe: Timeframe,
    current_candle: Option<OHLCV>,
}

impl CandleAggregator {
    pub fn new(timeframe: Timeframe) -> Self {
        Self {
            timeframe,
            current_candle: None,
        }
    }

    pub fn process_tick(&mut self, tick: &Tick) -> Option<OHLCV> {
        let tick_time = tick.timestamp;
        
        // If no current candle, start a new one
        if self.current_candle.is_none() {
            self.start_new_candle(tick);
            return None;
        }

        let mut completed_candle = None;

        if let Some(candle) = &mut self.current_candle {
            if tick_time >= candle.end_time {
                // Candle closed
                completed_candle = self.current_candle.take();
                self.start_new_candle(tick);
            } else {
                // Update current candle
                let price = (tick.bid + tick.ask) / Decimal::from(2);
                if price > candle.high {
                    candle.high = price;
                }
                if price < candle.low {
                    candle.low = price;
                }
                candle.close = price;
                
                // Volume is incremented by 1 tick unit for simplicity (or can be based on trade size if available)
                candle.volume += Decimal::from(1);
            }
        }

        completed_candle
    }

    fn start_new_candle(&mut self, tick: &Tick) {
        let price = (tick.bid + tick.ask) / Decimal::from(2);
        let start_time = self.align_timestamp(tick.timestamp);
        let end_time = start_time + self.timeframe_duration();

        self.current_candle = Some(OHLCV {
            open: price,
            high: price,
            low: price,
            close: price,
            volume: Decimal::from(1),
            start_time,
            end_time,
        });
    }

    fn align_timestamp(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        let ts_sec = timestamp.timestamp();
        let duration_sec = self.timeframe_duration().num_seconds();
        let aligned_sec = ts_sec - (ts_sec % duration_sec);
        DateTime::from_timestamp(aligned_sec, 0).unwrap_or(timestamp)
    }

    fn timeframe_duration(&self) -> Duration {
        match self.timeframe {
            Timeframe::M1 => Duration::minutes(1),
            Timeframe::M5 => Duration::minutes(5),
            Timeframe::M15 => Duration::minutes(15),
            Timeframe::M30 => Duration::minutes(30),
            Timeframe::H1 => Duration::hours(1),
            Timeframe::H4 => Duration::hours(4),
            Timeframe::D1 => Duration::days(1),
        }
    }
}

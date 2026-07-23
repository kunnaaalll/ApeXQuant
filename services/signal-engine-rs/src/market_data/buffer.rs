//! Timeframe-specific circular buffers for candle data

use dashmap::DashMap;
use std::collections::{HashMap, VecDeque};

use crate::config::Config;
use crate::market_data::candle::Candle;
use crate::Result;

/// Buffer for candles across all timeframes for a symbol
#[derive(Debug, Clone)]
pub struct TimeframeBuffer {
    /// Timeframe name (e.g., "M15")
    pub timeframe: String,
    /// Circular buffer of candles
    candles: VecDeque<Candle>,
    /// Maximum buffer size
    max_size: usize,
}

impl TimeframeBuffer {
    /// Create a new timeframe buffer
    pub fn new(timeframe: String, max_size: usize) -> Self {
        Self {
            timeframe,
            candles: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add candles to the buffer
    pub fn add_candles(&mut self, candles: Vec<Candle>) -> Result<()> {
        for candle in candles {
            // Remove candles exceeding max size
            if self.candles.len() >= self.max_size {
                self.candles.pop_front();
            }
            self.candles.push_back(candle);
        }
        Ok(())
    }

    /// Get all candles in the buffer
    pub fn get_candles(&self) -> Vec<Candle> {
        self.candles.iter().cloned().collect()
    }

    /// Get the most recent N candles
    pub fn recent(&self, n: usize) -> Vec<Candle> {
        let skip = self.candles.len().saturating_sub(n);
        self.candles.iter().skip(skip).cloned().collect()
    }

    /// Get the last candle
    pub fn last(&self) -> Option<&Candle> {
        self.candles.back()
    }

    /// Get buffer size
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }
}

/// Manages candle buffers for all symbols and timeframes
#[derive(Debug)]
pub struct CandleBuffer {
    /// Symbol -> Timeframe -> Buffer
    buffers: DashMap<String, HashMap<String, TimeframeBuffer>>,
    /// Configuration
    config: Config,
    /// Last update timestamp
    pub last_update: Option<time::OffsetDateTime>,
}

impl CandleBuffer {
    /// Create a new candle buffer manager
    pub fn new(config: &Config) -> Self {
        Self {
            buffers: DashMap::new(),
            config: config.clone(),
            last_update: None,
        }
    }

    /// Add candles for a specific symbol and timeframe
    pub fn add_candles(
        &mut self,
        symbol: &str,
        timeframe: &str,
        candles: Vec<Candle>,
    ) -> Result<()> {
        let mut entry = self.buffers.entry(symbol.to_string()).or_default();

        let buffer = entry
            .value_mut()
            .entry(timeframe.to_string())
            .or_insert_with(|| {
                TimeframeBuffer::new(timeframe.to_string(), self.get_buffer_size(timeframe))
            });

        let result = buffer.add_candles(candles);
        self.last_update = Some(time::OffsetDateTime::now_utc());
        result
    }

    /// Get candles for a specific symbol and timeframe
    pub fn get_candles(&self, symbol: &str, timeframe: &str) -> Result<Vec<Candle>> {
        match self.buffers.get(symbol) {
            Some(entry) => {
                let symbol_buffers = entry.value();
                match symbol_buffers.get(timeframe) {
                    Some(buffer) => Ok(buffer.recent(buffer.len())),
                    None => Err(crate::SignalEngineError::MissingTimeframe {
                        timeframe: timeframe.to_string(),
                    }),
                }
            }
            None => Err(crate::SignalEngineError::invalid_data(format!(
                "No data for symbol {}",
                symbol
            ))),
        }
    }

    /// Get all timeframe data for a symbol
    pub fn get_all_timeframes(&self, symbol: &str) -> Result<HashMap<String, Vec<Candle>>> {
        match self.buffers.get(symbol) {
            Some(entry) => {
                let symbol_buffers = entry.value();
                let mut result = HashMap::new();
                for (tf, buffer) in symbol_buffers {
                    result.insert(tf.clone(), buffer.recent(buffer.len()));
                }
                Ok(result)
            }
            None => Ok(HashMap::new()),
        }
    }

    /// Get the last candle across all timeframes for a symbol
    pub fn get_latest_candles(&self, symbol: &str) -> HashMap<String, Option<Candle>> {
        let mut result = HashMap::new();

        if let Some(entry) = self.buffers.get(symbol) {
            let symbol_buffers = entry.value();
            for (tf, buffer) in symbol_buffers {
                result.insert(tf.clone(), buffer.last().cloned());
            }
        }

        result
    }

    /// Clear all data for a symbol
    pub fn clear_symbol(&mut self, symbol: &str) {
        self.buffers.remove(symbol);
    }

    /// Get all active symbols
    pub fn get_symbols(&self) -> Vec<String> {
        self.buffers.iter().map(|e| e.key().clone()).collect()
    }

    /// Get buffer size based on timeframe
    fn get_buffer_size(&self, timeframe: &str) -> usize {
        // Larger buffers for higher timeframes
        match timeframe {
            "Monthly" | "Weekly" => 100,
            "Daily" => 200,
            "H4" => 500,
            "H1" => 1000,
            "M30" => 1000,
            "M15" => 1000,
            "M5" | "M1" => 500,
            _ => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn test_buffer_add_and_retrieve() {
        let mut buffer = TimeframeBuffer::new("M15".to_string(), 100);

        let candle = Candle::new(
            OffsetDateTime::now_utc(),
            rust_decimal::Decimal::new(100, 2),
            rust_decimal::Decimal::new(105, 2),
            rust_decimal::Decimal::new(98, 2),
            rust_decimal::Decimal::new(103, 2),
            1000,
        );

        buffer.add_candles(vec![candle.clone()]).unwrap();

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.last(), Some(&candle));
    }
}

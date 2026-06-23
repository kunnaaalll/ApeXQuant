use crate::tick::Tick;
use crate::streaming::TickStream;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

pub struct Mt5TickStream {
    _symbol: String,
    connected: bool,
    last_sequence: u64,
    last_timestamp: Option<DateTime<Utc>>,
    mock_queue: VecDeque<Tick>, // For testing purposes
}

impl Mt5TickStream {
    pub fn new(symbol: String) -> Self {
        Self {
            _symbol: symbol,
            connected: false,
            last_sequence: 0,
            last_timestamp: None,
            mock_queue: VecDeque::new(),
        }
    }

    pub fn inject_mock_tick(&mut self, tick: Tick) {
        self.mock_queue.push_back(tick);
    }
}

#[async_trait]
impl TickStream for Mt5TickStream {
    async fn connect(&mut self) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        Ok(())
    }

    async fn next_tick(&mut self) -> Option<Tick> {
        if !self.connected {
            return None;
        }

        let mut tick = self.mock_queue.pop_front()?;
        
        // Sequence assignment
        self.last_sequence += 1;
        tick.sequence = self.last_sequence;

        // Timestamp validation - ensure monotonically increasing or at least not strictly older than say 1 minute
        if let Some(last_ts) = self.last_timestamp {
            if tick.timestamp < last_ts {
                // Stale tick logic can be applied here.
                // We just pass it through and let the sequence/gap tracker handle it
            }
        }
        self.last_timestamp = Some(tick.timestamp);

        Some(tick)
    }
}

use crate::tick::Tick;
use std::collections::VecDeque;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaySpeed {
    Normal, // 1x
    Fast,   // 10x
    Rapid,  // 100x
    Turbo,  // 1000x
}

impl ReplaySpeed {
    pub fn multiplier(&self) -> u32 {
        match self {
            ReplaySpeed::Normal => 1,
            ReplaySpeed::Fast => 10,
            ReplaySpeed::Rapid => 100,
            ReplaySpeed::Turbo => 1000,
        }
    }
}

pub struct ReplayStream {
    ticks: VecDeque<Tick>,
    speed: ReplaySpeed,
}

impl ReplayStream {
    pub fn new(ticks: Vec<Tick>, speed: ReplaySpeed) -> Self {
        Self {
            ticks: VecDeque::from(ticks),
            speed,
        }
    }

    pub async fn next_tick(&mut self) -> Option<Tick> {
        let tick = self.ticks.pop_front()?;
        
        if let Some(next_tick) = self.ticks.front() {
            let delay = next_tick.timestamp.signed_duration_since(tick.timestamp).num_milliseconds();
            if delay > 0 {
                let adjusted_delay = (delay as u64) / (self.speed.multiplier() as u64);
                if adjusted_delay > 0 {
                    sleep(Duration::from_millis(adjusted_delay)).await;
                }
            }
        }
        
        Some(tick)
    }

    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }
}

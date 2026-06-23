pub struct StatisticsEngine {
    pub total_ticks: u64,
    pub duplicate_ticks: u64,
    pub gaps_detected: u64,
    pub candles_created: u64,
    pub replays_executed: u64,
}

impl Default for StatisticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsEngine {
    pub fn new() -> Self {
        Self {
            total_ticks: 0,
            duplicate_ticks: 0,
            gaps_detected: 0,
            candles_created: 0,
            replays_executed: 0,
        }
    }

    pub fn record_tick(&mut self) {
        self.total_ticks = self.total_ticks.saturating_add(1);
    }

    pub fn record_duplicate(&mut self) {
        self.duplicate_ticks = self.duplicate_ticks.saturating_add(1);
    }

    pub fn record_gap(&mut self) {
        self.gaps_detected = self.gaps_detected.saturating_add(1);
    }

    pub fn record_candle(&mut self) {
        self.candles_created = self.candles_created.saturating_add(1);
    }

    pub fn record_replay(&mut self) {
        self.replays_executed = self.replays_executed.saturating_add(1);
    }
}

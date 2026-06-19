use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAccumulator {
    pub wins: u64,
    pub losses: u64,
    pub expectancy_history: Vec<Decimal>,
    pub confidence_history: Vec<Decimal>,
    pub degradation_history: Vec<Decimal>,
}

impl EvidenceAccumulator {
    pub fn new() -> Self {
        Self {
            wins: 0,
            losses: 0,
            expectancy_history: Vec::new(),
            confidence_history: Vec::new(),
            degradation_history: Vec::new(),
        }
    }

    pub fn record_trade(&mut self, win: bool) {
        if win {
            self.wins += 1;
        } else {
            self.losses += 1;
        }
    }

    pub fn calculate_ema(data: &[Decimal], period: usize) -> Decimal {
        if data.is_empty() || period == 0 {
            return Decimal::from(0);
        }
        let alpha = Decimal::from(2) / Decimal::from((period + 1) as i64);
        let mut ema = data[0];

        for val in data.iter().skip(1) {
            ema = (val - ema) * alpha + ema;
        }
        ema
    }
}

impl Default for EvidenceAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct EvidenceAccumulator {
    pub successful_conditions_score: Decimal,
    pub poor_conditions_score: Decimal,
    pub decay_factor: Decimal,
}

impl EvidenceAccumulator {
    pub fn new(decay_factor: Decimal) -> Self {
        Self {
            successful_conditions_score: Decimal::ZERO,
            poor_conditions_score: Decimal::ZERO,
            decay_factor,
        }
    }

    pub fn record_success(&mut self, weight: Decimal) {
        self.successful_conditions_score =
            (self.successful_conditions_score * self.decay_factor) + weight;
        self.poor_conditions_score *= self.decay_factor;
    }

    pub fn record_failure(&mut self, weight: Decimal) {
        self.poor_conditions_score = (self.poor_conditions_score * self.decay_factor) + weight;
        self.successful_conditions_score *= self.decay_factor;
    }
}

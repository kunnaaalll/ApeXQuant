use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct TimeframeScore {
    pub combination: String, // e.g., "M15-H1"
    pub weight: Decimal,
    pub expectancy: Decimal,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct TimeframeOptimizer {
    pub min_evidence_trades: u32,
    pub max_weight_change: Decimal,
}

impl TimeframeOptimizer {
    pub fn new(min_evidence_trades: u32, max_weight_change: Decimal) -> Self {
        Self {
            min_evidence_trades,
            max_weight_change,
        }
    }

    pub fn evaluate(&self, combination: String, current_weight: Decimal, target_weight: Decimal, trades: u32, expectancy: Decimal) -> TimeframeScore {
        let is_valid = trades >= self.min_evidence_trades;
        
        let mut new_weight = current_weight;
        if is_valid {
            let diff = target_weight - current_weight;
            if diff > self.max_weight_change {
                new_weight += self.max_weight_change;
            } else if diff < -self.max_weight_change {
                new_weight -= self.max_weight_change;
            } else {
                new_weight = target_weight;
            }
        }

        TimeframeScore {
            combination,
            weight: new_weight,
            expectancy,
            is_valid,
        }
    }
}

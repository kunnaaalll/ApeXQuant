use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CorrelationGroups {
    // Maps pair of entities to their correlation coefficient (-1.0 to 1.0)
    pub correlations: HashMap<(String, String), Decimal>,
}

impl CorrelationGroups {
    pub fn new() -> Self {
        Self {
            correlations: HashMap::new(),
        }
    }

    pub fn record_correlation(&mut self, entity_a: String, entity_b: String, correlation: Decimal) {
        let key = if entity_a < entity_b {
            (entity_a, entity_b)
        } else {
            (entity_b, entity_a)
        };
        self.correlations.insert(key, correlation);
    }
}

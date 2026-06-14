use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct ConfidenceWeights {
    pub edge_weight: Decimal,
    pub expectancy_weight: Decimal,
    pub sample_weight: Decimal,
    pub regime_weight: Decimal,
    pub stability_weight: Decimal,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            edge_weight: dec!(0.25),
            expectancy_weight: dec!(0.25),
            sample_weight: dec!(0.2),
            regime_weight: dec!(0.15),
            stability_weight: dec!(0.15),
        }
    }
}

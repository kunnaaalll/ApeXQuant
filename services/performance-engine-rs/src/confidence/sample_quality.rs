use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleQualityState {
    Insufficient,
    Weak,
    Acceptable,
    Strong,
    Institutional,
}

#[derive(Debug, Clone)]
pub struct SampleQuality {
    pub trade_count: u32,
    pub state: SampleQualityState,
    pub confidence_multiplier: Decimal,
}

impl SampleQuality {
    pub fn evaluate(trade_count: u32) -> Self {
        let (state, multiplier) = if trade_count < 20 {
            (SampleQualityState::Insufficient, dec!(0.1))
        } else if trade_count < 50 {
            (SampleQualityState::Weak, dec!(0.5))
        } else if trade_count < 100 {
            (SampleQualityState::Acceptable, dec!(0.8))
        } else if trade_count < 300 {
            (SampleQualityState::Strong, dec!(1.0))
        } else {
            (SampleQualityState::Institutional, dec!(1.0))
        };

        Self {
            trade_count,
            state,
            confidence_multiplier: multiplier,
        }
    }
}

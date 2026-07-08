use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityResult {
    Perfect,
    Excellent,
    Acceptable,
    Poor,
    Failure,
}

pub struct ParityValidator;

impl ParityValidator {
    pub fn validate(
        execution_risk: Decimal,
        slippage: Decimal,
        latency: Decimal,
        liquidity: Decimal,
        fill_quality: Decimal,
        microstructure: Decimal,
        routing_state: Decimal,
    ) -> (ParityResult, Decimal) {
        let total = execution_risk
            + slippage
            + latency
            + liquidity
            + fill_quality
            + microstructure
            + routing_state;
        let count = dec!(7);
        let mut avg = total / count;

        if avg < dec!(0) {
            avg = dec!(0);
        } else if avg > dec!(100) {
            avg = dec!(100);
        }

        let result = if avg == dec!(100) {
            ParityResult::Perfect
        } else if avg >= dec!(90) {
            ParityResult::Excellent
        } else if avg >= dec!(75) {
            ParityResult::Acceptable
        } else if avg >= dec!(50) {
            ParityResult::Poor
        } else {
            ParityResult::Failure
        };

        (result, avg)
    }
}

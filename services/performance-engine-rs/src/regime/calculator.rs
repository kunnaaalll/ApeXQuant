use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use super::models::RegimeAssessment;
use super::states::RegimeState;
use super::types::{RegimeType, TradeCount};

pub struct RegimeCalculator;

impl RegimeCalculator {
    pub const MIN_TRADES_FOR_EVALUATION: TradeCount = 30;

    pub fn evaluate(
        regime: RegimeType,
        trade_count: TradeCount,
        wins: u32,
        losses: u32,
        expectancy: Decimal,
        profit_factor: Decimal,
        average_rr: Decimal,
        drawdown: Decimal,
        stability: Decimal,
    ) -> RegimeAssessment {
        let state = Self::determine_state(trade_count, expectancy, profit_factor);

        RegimeAssessment {
            regime,
            trade_count,
            wins,
            losses,
            expectancy,
            profit_factor,
            average_rr,
            drawdown,
            stability,
            state,
        }
    }

    fn determine_state(
        trade_count: TradeCount,
        expectancy: Decimal,
        profit_factor: Decimal,
    ) -> RegimeState {
        if trade_count < Self::MIN_TRADES_FOR_EVALUATION {
            return RegimeState::Normal; // Default until we have enough data
        }

        let one_point_five = Decimal::from_f64(1.5).unwrap();
        let two_point_zero = Decimal::from_f64(2.0).unwrap();
        let zero = Decimal::ZERO;

        if expectancy > zero && profit_factor >= two_point_zero {
            RegimeState::Exceptional
        } else if expectancy > zero && profit_factor >= one_point_five {
            RegimeState::Strong
        } else if expectancy > zero && profit_factor > Decimal::ONE {
            RegimeState::Normal
        } else if expectancy <= zero && profit_factor > zero {
            RegimeState::Weak
        } else {
            RegimeState::Negative
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_determine_state() {
        assert_eq!(
            RegimeCalculator::determine_state(10, dec!(10.0), dec!(2.5)),
            RegimeState::Normal // Insufficient trades
        );

        assert_eq!(
            RegimeCalculator::determine_state(35, dec!(10.0), dec!(2.5)),
            RegimeState::Exceptional
        );

        assert_eq!(
            RegimeCalculator::determine_state(35, dec!(5.0), dec!(1.6)),
            RegimeState::Strong
        );

        assert_eq!(
            RegimeCalculator::determine_state(35, dec!(-2.0), dec!(0.8)),
            RegimeState::Weak
        );
    }
}

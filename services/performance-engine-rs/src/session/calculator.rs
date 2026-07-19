use super::models::SessionAssessment;
use super::states::SessionState;
use super::types::{SessionType, TradeCount};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub struct SessionCalculator;

impl SessionCalculator {
    pub const MIN_TRADES_FOR_EVALUATION: TradeCount = 30;

    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        session: SessionType,
        trade_count: TradeCount,
        trade_frequency: Decimal,
        win_rate: Decimal,
        expectancy: Decimal,
        profit_factor: Decimal,
        average_rr: Decimal,
        drawdown: Decimal,
        stability: Decimal,
    ) -> SessionAssessment {
        let state = Self::determine_state(trade_count, expectancy, profit_factor, win_rate);

        SessionAssessment {
            session,
            trade_count,
            trade_frequency,
            win_rate,
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
        win_rate: Decimal,
    ) -> SessionState {
        if trade_count < Self::MIN_TRADES_FOR_EVALUATION {
            return SessionState::Normal; // Default until we have enough data
        }

        let one_point_five = Decimal::from_f64(1.5).unwrap_or_default();
        let two_point_zero = Decimal::from_f64(2.0).unwrap_or_default();
        let fifty_percent = Decimal::from_f64(0.50).unwrap_or_default();
        let zero = Decimal::ZERO;

        if expectancy > zero && profit_factor >= two_point_zero && win_rate >= fifty_percent {
            SessionState::Exceptional
        } else if expectancy > zero && profit_factor >= one_point_five {
            SessionState::Strong
        } else if expectancy > zero && profit_factor > Decimal::ONE {
            SessionState::Normal
        } else if expectancy <= zero && profit_factor > zero {
            SessionState::Weak
        } else {
            SessionState::Negative
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
            SessionCalculator::determine_state(10, dec!(10.0), dec!(2.5), dec!(0.6)),
            SessionState::Normal // Insufficient trades
        );

        assert_eq!(
            SessionCalculator::determine_state(35, dec!(10.0), dec!(2.5), dec!(0.6)),
            SessionState::Exceptional
        );

        assert_eq!(
            SessionCalculator::determine_state(35, dec!(5.0), dec!(1.6), dec!(0.4)),
            SessionState::Strong // exceptional needs win_rate >= 50%
        );

        assert_eq!(
            SessionCalculator::determine_state(35, dec!(-2.0), dec!(0.8), dec!(0.3)),
            SessionState::Weak
        );
    }
}

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use super::models::SymbolAssessment;
use super::states::SymbolState;
use super::types::{Symbol, TradeCount};

pub struct SymbolCalculator;

impl SymbolCalculator {
    pub const MIN_TRADES_FOR_EVALUATION: TradeCount = 30;

    pub fn evaluate(
        symbol: Symbol,
        trade_count: TradeCount,
        win_rate: Decimal,
        expectancy: Decimal,
        profit_factor: Decimal,
        average_rr: Decimal,
        drawdown: Decimal,
        stability: Decimal,
        edge_score: Decimal,
    ) -> SymbolAssessment {
        let state = Self::determine_state(trade_count, expectancy, profit_factor, edge_score);

        SymbolAssessment {
            symbol,
            trade_count,
            win_rate,
            expectancy,
            profit_factor,
            average_rr,
            drawdown,
            stability,
            edge_score,
            state,
        }
    }

    fn determine_state(
        trade_count: TradeCount,
        expectancy: Decimal,
        profit_factor: Decimal,
        edge_score: Decimal,
    ) -> SymbolState {
        // Small sample penalty
        if trade_count < Self::MIN_TRADES_FOR_EVALUATION {
            return SymbolState::Normal; // Default until sufficient data
        }

        let one_point_five = Decimal::from_f64(1.5).unwrap();
        let two_point_zero = Decimal::from_f64(2.0).unwrap();
        let high_edge = Decimal::from_f64(0.8).unwrap();
        let moderate_edge = Decimal::from_f64(0.5).unwrap();
        let zero = Decimal::ZERO;

        if expectancy > zero && profit_factor >= two_point_zero && edge_score >= high_edge {
            SymbolState::Exceptional
        } else if expectancy > zero && profit_factor >= one_point_five && edge_score >= moderate_edge {
            SymbolState::Strong
        } else if expectancy > zero && profit_factor > Decimal::ONE {
            SymbolState::Normal
        } else if expectancy <= zero && profit_factor > zero {
            SymbolState::Weak
        } else {
            SymbolState::Negative
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
            SymbolCalculator::determine_state(10, dec!(10.0), dec!(2.5), dec!(0.9)),
            SymbolState::Normal // Insufficient trades
        );

        assert_eq!(
            SymbolCalculator::determine_state(35, dec!(10.0), dec!(2.5), dec!(0.9)),
            SymbolState::Exceptional
        );

        assert_eq!(
            SymbolCalculator::determine_state(35, dec!(5.0), dec!(1.6), dec!(0.6)),
            SymbolState::Strong
        );

        assert_eq!(
            SymbolCalculator::determine_state(35, dec!(-2.0), dec!(0.8), dec!(0.1)),
            SymbolState::Weak
        );
    }
}

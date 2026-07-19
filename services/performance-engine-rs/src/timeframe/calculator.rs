use rust_decimal::Decimal;

use super::models::TimeframeAssessment;
use super::states::TimeframeState;
use super::types::{TimeframeType, TradeCount};

pub struct TimeframeCalculator;

pub struct TimeframeEvaluateParams {
    pub timeframe: TimeframeType,
    pub trade_count: TradeCount,
    pub trade_frequency: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub average_rr: Decimal,
    pub drawdown: Decimal,
    pub stability: Decimal,
    pub edge_score: Decimal,
}

impl TimeframeCalculator {
    pub const MIN_TRADES_FOR_EVALUATION: TradeCount = 30;

    pub fn evaluate(params: TimeframeEvaluateParams) -> TimeframeAssessment {
        let timeframe = params.timeframe;
        let trade_count = params.trade_count;
        let trade_frequency = params.trade_frequency;
        let expectancy = params.expectancy;
        let profit_factor = params.profit_factor;
        let average_rr = params.average_rr;
        let drawdown = params.drawdown;
        let stability = params.stability;
        let edge_score = params.edge_score;
        let state = Self::determine_state(trade_count, expectancy, profit_factor, edge_score);

        TimeframeAssessment {
            timeframe,
            trade_count,
            trade_frequency,
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
    ) -> TimeframeState {
        if trade_count < Self::MIN_TRADES_FOR_EVALUATION {
            return TimeframeState::Normal; // Default until we have enough data
        }

        let one_point_five = Decimal::new(15, 1);
        let two_point_zero = Decimal::new(20, 1);
        let high_edge = Decimal::new(8, 1);
        let moderate_edge = Decimal::new(5, 1);
        let zero = Decimal::ZERO;

        if expectancy > zero && profit_factor >= two_point_zero && edge_score >= high_edge {
            TimeframeState::Exceptional
        } else if expectancy > zero
            && profit_factor >= one_point_five
            && edge_score >= moderate_edge
        {
            TimeframeState::Strong
        } else if expectancy > zero && profit_factor > Decimal::ONE {
            TimeframeState::Normal
        } else if expectancy <= zero && profit_factor > zero {
            TimeframeState::Weak
        } else {
            TimeframeState::Negative
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
            TimeframeCalculator::determine_state(10, dec!(10.0), dec!(2.5), dec!(0.9)),
            TimeframeState::Normal // Insufficient trades
        );

        assert_eq!(
            TimeframeCalculator::determine_state(35, dec!(10.0), dec!(2.5), dec!(0.9)),
            TimeframeState::Exceptional
        );

        assert_eq!(
            TimeframeCalculator::determine_state(35, dec!(5.0), dec!(1.6), dec!(0.6)),
            TimeframeState::Strong
        );

        assert_eq!(
            TimeframeCalculator::determine_state(35, dec!(-2.0), dec!(0.8), dec!(0.1)),
            TimeframeState::Weak
        );
    }
}

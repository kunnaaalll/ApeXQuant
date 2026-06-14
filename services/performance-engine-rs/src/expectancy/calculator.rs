use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::models::ExpectancyMetrics;
use super::states::ExpectancyState;

pub struct ExpectancyCalculator;

impl ExpectancyCalculator {
    pub fn calculate(
        wins: u32,
        losses: u32,
        breakevens: u32,
        gross_profit: Decimal,
        gross_loss: Decimal, // Expecting positive or absolute representation
    ) -> ExpectancyMetrics {
        let trade_count = wins + losses + breakevens;
        
        if trade_count == 0 {
            return ExpectancyMetrics::default();
        }

        let total_trades_dec = Decimal::from(trade_count);
        let wins_dec = Decimal::from(wins);
        let losses_dec = Decimal::from(losses);

        let average_win = if wins > 0 {
            gross_profit / wins_dec
        } else {
            Decimal::ZERO
        };

        let average_loss = if losses > 0 {
            gross_loss / losses_dec
        } else {
            Decimal::ZERO
        };

        let win_rate = wins_dec / total_trades_dec;
        let loss_rate = losses_dec / total_trades_dec;

        let expectancy = (win_rate * average_win) - (loss_rate * average_loss);

        let profit_factor = if gross_loss.is_zero() {
            if gross_profit.is_zero() {
                Decimal::ZERO
            } else {
                dec!(999.99) // Bounded maximum
            }
        } else {
            gross_profit / gross_loss
        };

        let average_rr = if average_loss.is_zero() {
            if average_win.is_zero() {
                Decimal::ZERO
            } else {
                dec!(999.99)
            }
        } else {
            average_win / average_loss
        };

        ExpectancyMetrics {
            wins,
            losses,
            breakevens,
            average_win,
            average_loss,
            expectancy,
            profit_factor,
            average_rr,
            trade_count,
        }
    }

    pub fn determine_state(metrics: &ExpectancyMetrics) -> ExpectancyState {
        if metrics.trade_count < 30 {
            return ExpectancyState::Normal; // Too small sample size for strong claims
        }

        if metrics.expectancy > dec!(0.5) && metrics.profit_factor > dec!(2.0) {
            ExpectancyState::Exceptional
        } else if metrics.expectancy > dec!(0.2) && metrics.profit_factor > dec!(1.5) {
            ExpectancyState::Strong
        } else if metrics.expectancy > dec!(0.0) && metrics.profit_factor > dec!(1.0) {
            ExpectancyState::Normal
        } else if metrics.expectancy > dec!(-0.2) {
            ExpectancyState::Weak
        } else {
            ExpectancyState::Negative
        }
    }
}

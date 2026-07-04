use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct DrawdownCalculator;

impl DrawdownCalculator {
    /// Calculate maximum drawdown from equity curve
    pub fn calculate_max_drawdown(equity_curve: &[Decimal]) -> Decimal {
        if equity_curve.is_empty() {
            return Decimal::ZERO;
        }

        let mut peak = equity_curve[0];
        let mut max_dd = Decimal::ZERO;

        for &equity in equity_curve {
            if equity > peak {
                peak = equity;
            } else if peak > Decimal::ZERO {
                let dd = (peak - equity) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }
        }

        max_dd
    }
}

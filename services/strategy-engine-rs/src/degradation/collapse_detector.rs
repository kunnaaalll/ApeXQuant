use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct CollapseDetector;

impl CollapseDetector {
    /// Detect collapse if drawdown exceeds limit or profit factor falls below threshold.
    pub fn is_collapsed(
        current_drawdown: Decimal,
        max_drawdown_limit: Decimal,
        profit_factor: Decimal,
    ) -> bool {
        if current_drawdown >= max_drawdown_limit {
            return true;
        }

        if profit_factor < dec!(0.8) {
            return true;
        }

        false
    }
}

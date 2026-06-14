use rust_decimal::Decimal;

pub struct RealizedPnL;

impl RealizedPnL {
    /// Calculates realized PnL for a closing transaction.
    /// Assuming Long: (exit_price - entry_price) * closed_size.
    pub fn calculate(exit_price: Decimal, average_entry: Decimal, closed_size: Decimal) -> Decimal {
        (exit_price - average_entry) * closed_size
    }
}

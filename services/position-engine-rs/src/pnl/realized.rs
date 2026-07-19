use rust_decimal::Decimal;

pub struct RealizedPnL;

impl RealizedPnL {
    /// Calculates realized PnL for a closing transaction.
    pub fn calculate(
        side: &str,
        exit_price: Decimal,
        average_entry: Decimal,
        closed_size: Decimal,
    ) -> Decimal {
        if side.eq_ignore_ascii_case("buy") {
            (exit_price - average_entry) * closed_size
        } else {
            (average_entry - exit_price) * closed_size
        }
    }
}

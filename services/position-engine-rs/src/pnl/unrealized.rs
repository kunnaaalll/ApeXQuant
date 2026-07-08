use rust_decimal::Decimal;

pub struct UnrealizedPnL;

impl UnrealizedPnL {
    /// Calculates unrealized PnL based on position direction.
    pub fn calculate(
        side: &str,
        current_price: Decimal,
        average_entry: Decimal,
        current_size: Decimal,
    ) -> Decimal {
        if side.eq_ignore_ascii_case("buy") {
            (current_price - average_entry) * current_size
        } else {
            (average_entry - current_price) * current_size
        }
    }
}

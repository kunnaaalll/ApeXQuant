use rust_decimal::Decimal;

pub struct UnrealizedPnL;

impl UnrealizedPnL {
    /// Calculates unrealized PnL based on position direction (Long vs Short assumes handled via signed size or explicit flag).
    /// For this simple baseline, assuming Long: (current - entry) * size.
    pub fn calculate(
        current_price: Decimal,
        average_entry: Decimal,
        current_size: Decimal,
    ) -> Decimal {
        (current_price - average_entry) * current_size
    }
}

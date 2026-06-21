use rust_decimal::Decimal;

#[derive(Debug, Clone, Default)]
pub struct FillStatistics {
    pub total_fills: usize,
    pub total_quantity_filled: Decimal,
}

impl FillStatistics {
    pub fn new() -> Self {
        Self::default()
    }
}

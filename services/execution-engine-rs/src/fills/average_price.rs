use rust_decimal::Decimal;

#[derive(Debug, Clone, Default)]
pub struct AveragePriceCalculator {
    total_value: Decimal,
    total_quantity: Decimal,
}

impl AveragePriceCalculator {
    pub fn new() -> Self {
        Self {
            total_value: Decimal::ZERO,
            total_quantity: Decimal::ZERO,
        }
    }

    pub fn add_fill(&mut self, price: Decimal, quantity: Decimal) {
        self.total_value += price * quantity;
        self.total_quantity += quantity;
    }

    pub fn average_price(&self) -> Option<Decimal> {
        if self.total_quantity == Decimal::ZERO {
            None
        } else {
            Some((self.total_value / self.total_quantity).trunc_with_scale(8))
        }
    }
}

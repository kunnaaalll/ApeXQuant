use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillState {
    None,
    Partial,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialFillEngine {
    pub requested_quantity: Decimal,
    pub filled_quantity: Decimal,
}

impl PartialFillEngine {
    pub fn new(requested_quantity: Decimal) -> Self {
        Self {
            requested_quantity,
            filled_quantity: Decimal::ZERO,
        }
    }

    pub fn add_fill(&mut self, quantity: Decimal) -> Result<FillState, &'static str> {
        if quantity <= Decimal::ZERO {
            return Err("Fill quantity must be positive");
        }

        let new_filled = self.filled_quantity + quantity;
        if new_filled > self.requested_quantity {
            return Err("Fill quantity exceeds requested quantity");
        }

        self.filled_quantity = new_filled;
        Ok(self.state())
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.requested_quantity - self.filled_quantity
    }

    pub fn state(&self) -> FillState {
        if self.filled_quantity == Decimal::ZERO {
            FillState::None
        } else if self.filled_quantity < self.requested_quantity {
            FillState::Partial
        } else {
            FillState::Completed
        }
    }
}

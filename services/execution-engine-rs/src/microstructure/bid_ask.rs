use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BidAsk {
    pub bid: Decimal,
    pub ask: Decimal,
}

impl BidAsk {
    pub fn new(bid: Decimal, ask: Decimal) -> Result<Self, &'static str> {
        if bid >= ask {
            return Err("Bid must be strictly less than ask");
        }
        if bid < Decimal::ZERO || ask < Decimal::ZERO {
            return Err("Prices must be positive");
        }
        Ok(Self { bid, ask })
    }

    pub fn spread(&self) -> Decimal {
        self.ask - self.bid
    }

    pub fn midpoint(&self) -> Decimal {
        (self.bid + self.ask) / Decimal::new(2, 0)
    }
}

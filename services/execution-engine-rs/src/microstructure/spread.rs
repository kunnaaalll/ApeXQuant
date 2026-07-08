use crate::microstructure::bid_ask::BidAsk;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spread {
    pub absolute: Decimal,
    pub relative: Decimal, // bps
}

impl Spread {
    pub fn calculate(bid_ask: &BidAsk) -> Result<Self, &'static str> {
        let absolute = bid_ask.spread();
        let mid = bid_ask.midpoint();
        if mid == Decimal::ZERO {
            return Err("Midpoint is zero, cannot calculate relative spread");
        }
        let relative = (absolute / mid) * Decimal::new(10000, 0); // Convert to bps

        Ok(Self { absolute, relative })
    }
}

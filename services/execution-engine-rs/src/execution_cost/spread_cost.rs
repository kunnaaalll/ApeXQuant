use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpreadCost {
    pub cost_usd: Decimal,
}

impl SpreadCost {
    pub fn calculate(spread_bps: Decimal, notional_usd: Decimal) -> Result<Self, &'static str> {
        if notional_usd < Decimal::ZERO || spread_bps < Decimal::ZERO {
            return Err("Values cannot be negative");
        }

        // spread_bps is in basis points, so divide by 10000
        let cost_usd = notional_usd * spread_bps / Decimal::new(10000, 0);
        Ok(Self { cost_usd })
    }
}

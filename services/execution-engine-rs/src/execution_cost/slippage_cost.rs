use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlippageCost {
    pub cost_usd: Decimal,
}

impl SlippageCost {
    pub fn calculate(slippage_bps: Decimal, notional_usd: Decimal) -> Result<Self, &'static str> {
        if notional_usd < Decimal::ZERO || slippage_bps < Decimal::ZERO {
            return Err("Values cannot be negative");
        }

        let cost_usd = notional_usd * slippage_bps / Decimal::new(10000, 0);
        Ok(Self { cost_usd })
    }
}

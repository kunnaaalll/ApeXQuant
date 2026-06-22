use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactCost {
    pub cost_usd: Decimal,
}

impl ImpactCost {
    pub fn calculate(impact_bps: Decimal, notional_usd: Decimal) -> Result<Self, &'static str> {
        if notional_usd < Decimal::ZERO || impact_bps < Decimal::ZERO {
            return Err("Values cannot be negative");
        }

        let cost_usd = notional_usd * impact_bps / Decimal::new(10000, 0);
        Ok(Self { cost_usd })
    }
}

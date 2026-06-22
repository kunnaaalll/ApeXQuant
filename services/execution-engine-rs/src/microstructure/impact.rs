use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketImpactGrade {
    Negligible,
    Low,
    Moderate,
    High,
    Extreme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketImpact {
    pub grade: MarketImpactGrade,
    pub score: u8, // 0-100
}

impl MarketImpact {
    pub fn calculate(expected_slippage_bps: Decimal) -> Result<Self, &'static str> {
        if expected_slippage_bps < Decimal::ZERO {
            return Err("Slippage cannot be negative");
        }

        use rust_decimal::prelude::ToPrimitive;
        let slippage_u64 = expected_slippage_bps.to_u64().unwrap_or(100);
        
        let (grade, score) = if slippage_u64 <= 1 {
            (MarketImpactGrade::Negligible, 0)
        } else if slippage_u64 <= 5 {
            (MarketImpactGrade::Low, 25)
        } else if slippage_u64 <= 15 {
            (MarketImpactGrade::Moderate, 50)
        } else if slippage_u64 <= 30 {
            (MarketImpactGrade::High, 75)
        } else {
            (MarketImpactGrade::Extreme, 100)
        };

        Ok(Self { grade, score })
    }
}

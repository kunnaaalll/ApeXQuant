use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleBias {
    pub multiplier: Decimal,
}

impl SampleBias {
    pub fn calculate(trades: u32) -> Self {
        let exact_multiplier = if trades < 20 {
            Decimal::new(20, 2) // 0.20
        } else if trades < 50 {
            Decimal::new(40, 2) // 0.40
        } else if trades < 100 {
            Decimal::new(60, 2) // 0.60
        } else if trades < 300 {
            Decimal::new(80, 2) // 0.80
        } else {
            Decimal::new(100, 2) // 1.00
        };

        Self {
            multiplier: exact_multiplier,
        }
    }
}

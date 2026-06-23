use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolatilityGrade {
    VeryLow,
    Low,
    Normal,
    High,
    Extreme,
}

pub struct VolatilityMetrics {
    pub price_range: Decimal,
    pub atr: Decimal, // simple atr proxy
    pub grade: VolatilityGrade,
}

pub struct VolatilityEngine;

impl VolatilityEngine {
    pub fn calculate(high: Decimal, low: Decimal, previous_close: Decimal) -> Result<VolatilityMetrics, &'static str> {
        if high < low {
            return Err("High cannot be less than low");
        }

        let price_range = high - low;
        
        let tr1 = price_range;
        let tr2 = if previous_close.is_zero() { Decimal::ZERO } else { (high - previous_close).abs() };
        let tr3 = if previous_close.is_zero() { Decimal::ZERO } else { (low - previous_close).abs() };

        let true_range = tr1.max(tr2).max(tr3);
        
        let atr = true_range;

        let relative_vol = if previous_close.is_zero() {
            Decimal::ZERO
        } else {
            (atr / previous_close) * Decimal::from(10000) // bps
        };

        let grade = match relative_vol {
            v if v < Decimal::from(10) => VolatilityGrade::VeryLow,
            v if v < Decimal::from(50) => VolatilityGrade::Low,
            v if v < Decimal::from(150) => VolatilityGrade::Normal,
            v if v < Decimal::from(300) => VolatilityGrade::High,
            _ => VolatilityGrade::Extreme,
        };

        Ok(VolatilityMetrics {
            price_range,
            atr,
            grade,
        })
    }
}

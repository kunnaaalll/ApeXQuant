use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImbalanceGrade {
    Balanced,
    BuyPressure,
    SellPressure,
    ExtremeBuy,
    ExtremeSell,
}

pub struct ImbalanceMetrics {
    pub buy_ratio: Decimal,
    pub grade: ImbalanceGrade,
}

pub struct ImbalanceEngine;

impl ImbalanceEngine {
    pub fn calculate(buy_volume: Decimal, sell_volume: Decimal) -> Result<ImbalanceMetrics, &'static str> {
        if buy_volume < Decimal::ZERO || sell_volume < Decimal::ZERO {
            return Err("Volume cannot be negative");
        }

        let total = buy_volume + sell_volume;
        if total.is_zero() {
            return Ok(ImbalanceMetrics {
                buy_ratio: Decimal::new(5, 1), // 0.5
                grade: ImbalanceGrade::Balanced,
            });
        }
        
        let buy_ratio = buy_volume / total;
        
        let grade = match buy_ratio {
            v if v > Decimal::new(80, 2) => ImbalanceGrade::ExtremeBuy,   // > 0.8
            v if v > Decimal::new(60, 2) => ImbalanceGrade::BuyPressure,  // > 0.6
            v if v < Decimal::new(20, 2) => ImbalanceGrade::ExtremeSell,  // < 0.2
            v if v < Decimal::new(40, 2) => ImbalanceGrade::SellPressure, // < 0.4
            _ => ImbalanceGrade::Balanced,
        };

        Ok(ImbalanceMetrics {
            buy_ratio,
            grade,
        })
    }
}

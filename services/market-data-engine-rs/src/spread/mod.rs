use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpreadGrade {
    Elite,
    Strong,
    Normal,
    Weak,
    Poor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpreadMetrics {
    pub absolute_spread: Decimal,
    pub relative_spread_bps: Decimal,
    pub grade: SpreadGrade,
}

pub struct SpreadEngine;

impl SpreadEngine {
    pub fn calculate(bid: Decimal, ask: Decimal) -> Result<SpreadMetrics, &'static str> {
        if bid > ask {
            return Err("Bid must not be greater than ask");
        }
        if ask.is_zero() {
            return Err("Ask cannot be zero");
        }

        let absolute_spread = ask - bid;
        
        let mut relative_spread_bps = (absolute_spread / ask) * Decimal::from(10000);
        let max_bps = Decimal::from(10000);
        if relative_spread_bps > max_bps {
            relative_spread_bps = max_bps;
        }

        let grade = match relative_spread_bps {
            v if v < Decimal::from(1) => SpreadGrade::Elite, // < 1 bps
            v if v < Decimal::from(5) => SpreadGrade::Strong, // < 5 bps
            v if v < Decimal::from(15) => SpreadGrade::Normal, // < 15 bps
            v if v < Decimal::from(50) => SpreadGrade::Weak, // < 50 bps
            _ => SpreadGrade::Poor,
        };

        Ok(SpreadMetrics {
            absolute_spread,
            relative_spread_bps,
            grade,
        })
    }
}

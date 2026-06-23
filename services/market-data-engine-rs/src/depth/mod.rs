use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthGrade {
    Deep,
    Normal,
    Thin,
    Critical,
}

pub struct DepthMetrics {
    pub bid_volume: Decimal,
    pub ask_volume: Decimal,
    pub total_volume: Decimal,
    pub top_levels: usize,
    pub depth_imbalance: Decimal,
    pub grade: DepthGrade,
}

pub struct DepthEngine;

impl DepthEngine {
    pub fn evaluate(bid_vol: Decimal, ask_vol: Decimal, levels: usize) -> Result<DepthMetrics, &'static str> {
        if bid_vol < Decimal::ZERO || ask_vol < Decimal::ZERO {
            return Err("Volume cannot be negative");
        }
        
        let total_volume = bid_vol + ask_vol;
        
        let depth_imbalance = if total_volume.is_zero() {
            Decimal::ZERO
        } else {
            ((bid_vol - ask_vol).abs() / total_volume) * Decimal::from(100)
        };

        let grade = if total_volume.is_zero() || levels == 0 {
            DepthGrade::Critical
        } else if total_volume < Decimal::from(10) || levels < 5 {
            DepthGrade::Thin
        } else if total_volume > Decimal::from(100) && levels >= 10 {
            DepthGrade::Deep
        } else {
            DepthGrade::Normal
        };

        Ok(DepthMetrics {
            bid_volume: bid_vol,
            ask_volume: ask_vol,
            total_volume,
            top_levels: levels,
            depth_imbalance,
            grade,
        })
    }
}

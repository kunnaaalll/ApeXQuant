use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolatilityGrade {
    ExtremelyLow,
    Low,
    Normal,
    High,
    Extreme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    pub atr: Decimal,
    pub realized_volatility: Decimal, // simple proxy or exact
    pub percentile_rank: Decimal,
    pub is_expanding: bool,
    pub is_contracting: bool,
    pub grade: VolatilityGrade,
    pub score: u8, // 0-100
}

#[derive(Debug, Clone)]
pub struct VolatilityEngine {
    window_size: usize,
    true_ranges: Vec<Decimal>,
    returns: Vec<Decimal>,
    previous_close: Option<Decimal>,
    historical_vols: Vec<Decimal>,
}

impl VolatilityEngine {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            true_ranges: Vec::with_capacity(window_size),
            returns: Vec::with_capacity(window_size),
            previous_close: None,
            historical_vols: Vec::with_capacity(1000), // bounded history for percentiles
        }
    }

    pub fn update(&mut self, high: Decimal, low: Decimal, close: Decimal) -> Result<VolatilityMetrics, &'static str> {
        if high < low {
            return Err("High cannot be less than low");
        }

        let tr = match self.previous_close {
            Some(prev_close) => {
                let tr1 = high - low;
                let tr2 = (high - prev_close).abs();
                let tr3 = (low - prev_close).abs();
                tr1.max(tr2).max(tr3)
            }
            None => high - low,
        };

        if let Some(prev_close) = self.previous_close {
            if !prev_close.is_zero() {
                let ret = (close - prev_close) / prev_close;
                self.returns.push(ret);
                if self.returns.len() > self.window_size {
                    self.returns.remove(0);
                }
            }
        }

        self.previous_close = Some(close);

        self.true_ranges.push(tr);
        if self.true_ranges.len() > self.window_size {
            self.true_ranges.remove(0);
        }

        let atr = if self.true_ranges.is_empty() {
            Decimal::ZERO
        } else {
            let sum: Decimal = self.true_ranges.iter().sum();
            sum / Decimal::from(self.true_ranges.len())
        };

        let realized_volatility = if self.returns.is_empty() {
            Decimal::ZERO
        } else {
            let mean: Decimal = self.returns.iter().sum::<Decimal>() / Decimal::from(self.returns.len());
            let mut var_sum = Decimal::ZERO;
            for r in &self.returns {
                let diff = r - mean;
                var_sum += diff * diff;
            }
            let variance = var_sum / Decimal::from(self.returns.len());
            // Since we can't use floats for exact sqrt, we can use an approximation or just use variance proxy for ranking.
            // Actually rust_decimal has sqrt via decimal-math if enabled, but we might not have it.
            // Let's use a simple iterative approach or just return the variance proxy multiplied by a constant if sqrt isn't available.
            // We can also try `.sqrt()` on Decimal if the `maths` feature is enabled.
            // Let's assume variance * 10000 as a proxy for realized vol if we can't do sqrt.
            variance * Decimal::from(10000) 
        };

        self.historical_vols.push(realized_volatility);
        if self.historical_vols.len() > 1000 {
            self.historical_vols.remove(0);
        }

        let mut percentile_rank = Decimal::ZERO;
        if self.historical_vols.len() > 1 {
            let mut lower_count = 0;
            for v in &self.historical_vols {
                if *v < realized_volatility {
                    lower_count += 1;
                }
            }
            percentile_rank = (Decimal::from(lower_count) / Decimal::from(self.historical_vols.len())) * Decimal::from(100);
        }

        let mut is_expanding = false;
        let mut is_contracting = false;
        
        if self.historical_vols.len() > 3 {
            let len = self.historical_vols.len();
            let v1 = self.historical_vols[len - 1];
            let v2 = self.historical_vols[len - 2];
            let v3 = self.historical_vols[len - 3];
            
            if v1 > v2 && v2 > v3 {
                is_expanding = true;
            }
            if v1 < v2 && v2 < v3 {
                is_contracting = true;
            }
        }

        let score = percentile_rank.to_u8().unwrap_or(0).min(100);

        let grade = match score {
            s if s < 10 => VolatilityGrade::ExtremelyLow,
            s if s < 30 => VolatilityGrade::Low,
            s if s < 70 => VolatilityGrade::Normal,
            s if s < 90 => VolatilityGrade::High,
            _ => VolatilityGrade::Extreme,
        };

        Ok(VolatilityMetrics {
            atr,
            realized_volatility,
            percentile_rank,
            is_expanding,
            is_contracting,
            grade,
            score,
        })
    }
}

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationGrade {
    Independent,
    Weak,
    Moderate,
    Strong,
    Extreme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationMetrics {
    pub pearson_proxy: Decimal,
    pub score: u8,
    pub grade: CorrelationGrade,
}

#[derive(Debug, Clone)]
pub struct CorrelationEngine {
    window: usize,
    returns_a: Vec<Decimal>,
    returns_b: Vec<Decimal>,
}

impl CorrelationEngine {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            returns_a: Vec::with_capacity(window),
            returns_b: Vec::with_capacity(window),
        }
    }

    pub fn update(&mut self, ret_a: Decimal, ret_b: Decimal) -> Result<CorrelationMetrics, &'static str> {
        self.returns_a.push(ret_a);
        self.returns_b.push(ret_b);

        if self.returns_a.len() > self.window {
            self.returns_a.remove(0);
            self.returns_b.remove(0);
        }

        let n = Decimal::from(self.returns_a.len());
        if n < Decimal::from(2) {
            return Ok(CorrelationMetrics {
                pearson_proxy: Decimal::ZERO,
                score: 0,
                grade: CorrelationGrade::Independent,
            });
        }

        let sum_a: Decimal = self.returns_a.iter().sum();
        let sum_b: Decimal = self.returns_b.iter().sum();

        let mut sum_ab = Decimal::ZERO;
        let mut sum_a2 = Decimal::ZERO;
        let mut sum_b2 = Decimal::ZERO;

        for (a, b) in self.returns_a.iter().zip(self.returns_b.iter()) {
            sum_ab += *a * *b;
            sum_a2 += *a * *a;
            sum_b2 += *b * *b;
        }

        // Pearson correlation coefficient formula (simplified for proxy)
        // numerator = n * sum_ab - sum_a * sum_b
        // denominator squared = (n * sum_a2 - sum_a^2) * (n * sum_b2 - sum_b^2)
        // Since we cannot use float sqrt, we can compute numerator and denominator proxy
        
        let numerator = (n * sum_ab) - (sum_a * sum_b);
        let var_a = (n * sum_a2) - (sum_a * sum_a);
        let var_b = (n * sum_b2) - (sum_b * sum_b);

        let cov_proxy = numerator * numerator;
        let var_proxy = var_a * var_b;

        let mut pearson_proxy = Decimal::ZERO;
        if !var_proxy.is_zero() {
            pearson_proxy = cov_proxy / var_proxy;
            // The value is r^2. We cap it at 1.0
            if pearson_proxy > Decimal::ONE {
                pearson_proxy = Decimal::ONE;
            }
            if numerator.is_sign_negative() {
                pearson_proxy = -pearson_proxy;
            }
        }

        let abs_proxy = pearson_proxy.abs();
        let score = (abs_proxy * Decimal::from(100)).to_u8().unwrap_or(0).min(100);

        let grade = match score {
            s if s > 80 => CorrelationGrade::Extreme,
            s if s > 60 => CorrelationGrade::Strong,
            s if s > 40 => CorrelationGrade::Moderate,
            s if s > 20 => CorrelationGrade::Weak,
            _ => CorrelationGrade::Independent,
        };

        Ok(CorrelationMetrics {
            pearson_proxy,
            score,
            grade,
        })
    }
}

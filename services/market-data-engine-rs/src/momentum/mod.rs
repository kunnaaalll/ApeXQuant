use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MomentumState {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MomentumGrade {
    Strong,
    Normal,
    Weak,
}

pub struct MomentumMetrics {
    pub score: u32,
    pub state: MomentumState,
    pub grade: MomentumGrade,
}

pub struct MomentumEngine;

impl MomentumEngine {
    pub fn calculate(current_price: Decimal, previous_price: Decimal, rsi_proxy: Decimal) -> Result<MomentumMetrics, &'static str> {
        if current_price < Decimal::ZERO || previous_price < Decimal::ZERO {
            return Err("Prices cannot be negative");
        }
        
        let state = if current_price > previous_price {
            MomentumState::Positive
        } else if current_price < previous_price {
            MomentumState::Negative
        } else {
            MomentumState::Neutral
        };

        let score = match rsi_proxy.to_u32() {
            Some(s) => s.clamp(0, 100),
            None => 50,
        };

        let grade = match score {
            s if s >= 70 || s <= 30 => MomentumGrade::Strong,
            s if s > 45 && s < 55 => MomentumGrade::Weak,
            _ => MomentumGrade::Normal,
        };

        Ok(MomentumMetrics {
            score,
            state,
            grade,
        })
    }
}

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExecutionGrade {
    Poor,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExecutionScoreError {
    #[error("Score components must be between 0 and 100")]
    OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionScore {
    pub slippage_score: Decimal,
    pub fill_quality: Decimal,
    pub liquidity_quality: Decimal,
    pub latency_score: Decimal,
    pub final_score: Decimal,
}

impl ExecutionScore {
    pub fn new(
        slippage_score: Decimal,
        fill_quality: Decimal,
        liquidity_quality: Decimal,
        latency_score: Decimal,
    ) -> Result<Self, ExecutionScoreError> {
        let zero = dec!(0);
        let hundred = dec!(100);

        let validate = |d: Decimal| -> bool { d >= zero && d <= hundred };

        if !validate(slippage_score)
            || !validate(fill_quality)
            || !validate(liquidity_quality)
            || !validate(latency_score)
        {
            return Err(ExecutionScoreError::OutOfBounds);
        }

        // Default balanced weights: 40% slippage, 30% fill, 20% liquidity, 10% latency
        let final_score = (slippage_score * dec!(0.40))
            + (fill_quality * dec!(0.30))
            + (liquidity_quality * dec!(0.20))
            + (latency_score * dec!(0.10));

        let mut score = final_score;
        if score > hundred {
            score = hundred;
        } else if score < zero {
            score = zero;
        }

        Ok(Self {
            slippage_score,
            fill_quality,
            liquidity_quality,
            latency_score,
            final_score: score,
        })
    }

    pub fn grade(&self) -> ExecutionGrade {
        if self.final_score >= dec!(90) {
            ExecutionGrade::Elite
        } else if self.final_score >= dec!(75) {
            ExecutionGrade::Strong
        } else if self.final_score >= dec!(50) {
            ExecutionGrade::Normal
        } else if self.final_score >= dec!(25) {
            ExecutionGrade::Weak
        } else {
            ExecutionGrade::Poor
        }
    }
}

use super::{calculate_score, grade_from_score, RankingGrade};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeframeOptimizer {
    score: Decimal,
    grade: RankingGrade,
}

impl TimeframeOptimizer {
    pub fn new() -> Self {
        Self {
            score: rust_decimal_macros::dec!(0.0),
            grade: RankingGrade::Forbidden,
        }
    }

    pub fn optimize(
        &mut self,
        expectancy: Decimal,
        confidence: Decimal,
        stability: Decimal,
        drawdown: Decimal,
        sample_quality: Decimal,
    ) {
        self.score = calculate_score(expectancy, confidence, stability, drawdown, sample_quality);
        self.grade = grade_from_score(self.score);
    }

    pub fn score(&self) -> Decimal {
        self.score
    }

    pub fn grade(&self) -> RankingGrade {
        self.grade
    }
}

impl Default for TimeframeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

pub mod events;
pub mod snapshots;

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternAssessment {
    Exceptional,
    Strong,
    Normal,
    Weak,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    IncreaseAllocation,
    Maintain,
    ReduceAllocation,
    Pause,
    Research,
    Disable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeIntelligence {
    pub expectancy: Decimal,
    pub win_rate: Decimal,
    pub rr: Decimal,
    pub stability: Decimal,
    pub drawdown: Decimal,
}

impl EdgeIntelligence {
    pub fn new(
        expectancy: Decimal,
        win_rate: Decimal,
        rr: Decimal,
        stability: Decimal,
        drawdown: Decimal,
    ) -> Self {
        Self {
            expectancy,
            win_rate,
            rr,
            stability,
            drawdown,
        }
    }

    pub fn assess(&self) -> PatternAssessment {
        let exceptional_threshold = Decimal::new(5, 1); // 0.5
        let strong_threshold = Decimal::new(2, 1); // 0.2
        let weak_threshold = Decimal::new(-2, 1); // -0.2

        if self.expectancy >= exceptional_threshold && self.stability >= Decimal::new(8, 1) {
            PatternAssessment::Exceptional
        } else if self.expectancy >= strong_threshold {
            PatternAssessment::Strong
        } else if self.expectancy > Decimal::from(0) {
            PatternAssessment::Normal
        } else if self.expectancy > weak_threshold {
            PatternAssessment::Weak
        } else {
            PatternAssessment::Negative
        }
    }

    pub fn recommend(&self) -> Recommendation {
        match self.assess() {
            PatternAssessment::Exceptional => Recommendation::IncreaseAllocation,
            PatternAssessment::Strong => Recommendation::IncreaseAllocation,
            PatternAssessment::Normal => Recommendation::Maintain,
            PatternAssessment::Weak => Recommendation::ReduceAllocation,
            PatternAssessment::Negative => Recommendation::Pause,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectancyAssessment {
    pub quality: Decimal,
    pub degradation: Decimal,
    pub acceleration: Decimal,
}

impl ExpectancyAssessment {
    pub fn new(quality: Decimal, degradation: Decimal, acceleration: Decimal) -> Self {
        Self {
            quality,
            degradation,
            acceleration,
        }
    }
}

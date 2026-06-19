use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeframeGrade {
    Forbidden,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeframeAssessment {
    pub expectancy: Decimal,
    pub confidence: Decimal,
    pub stability: Decimal,
}

impl TimeframeAssessment {
    pub fn grade(&self) -> TimeframeGrade {
        let score = self.expectancy * self.confidence * self.stability;

        if score >= Decimal::from(1000) {
            TimeframeGrade::Elite
        } else if score >= Decimal::from(500) {
            TimeframeGrade::Strong
        } else if score >= Decimal::from(100) {
            TimeframeGrade::Normal
        } else if score >= Decimal::from(10) {
            TimeframeGrade::Weak
        } else {
            TimeframeGrade::Forbidden
        }
    }
}

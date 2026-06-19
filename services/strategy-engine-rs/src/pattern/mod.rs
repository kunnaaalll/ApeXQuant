use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternGrade {
    Negative,
    Weak,
    Normal,
    Strong,
    Exceptional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternAssessment {
    pub setup_expectancy: Decimal,
    pub rr: Decimal,
    pub confidence: Decimal,
    pub sample_quality: Decimal,
}

impl PatternAssessment {
    pub fn grade(&self) -> PatternGrade {
        let score = self.setup_expectancy * self.rr * self.confidence * self.sample_quality;

        if score >= Decimal::from(1000) {
            PatternGrade::Exceptional
        } else if score >= Decimal::from(500) {
            PatternGrade::Strong
        } else if score >= Decimal::from(100) {
            PatternGrade::Normal
        } else if score >= Decimal::from(10) {
            PatternGrade::Weak
        } else {
            PatternGrade::Negative
        }
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Session {
    Asia,
    London,
    NewYork,
    LondonNewYorkOverlap,
    AsiaLondonOverlap,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionGrade {
    Negative,
    Weak,
    Normal,
    Strong,
    Exceptional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionAssessment {
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub edge: Decimal,
    pub confidence: Decimal,
    pub degradation: Decimal,
}

impl SessionAssessment {
    pub fn grade(&self) -> SessionGrade {
        let score =
            (self.win_rate * self.expectancy * self.edge * self.confidence) - self.degradation;

        if score >= Decimal::from(1000) {
            SessionGrade::Exceptional
        } else if score >= Decimal::from(500) {
            SessionGrade::Strong
        } else if score >= Decimal::from(100) {
            SessionGrade::Normal
        } else if score >= Decimal::from(10) {
            SessionGrade::Weak
        } else {
            SessionGrade::Negative
        }
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Regime {
    Trending,
    Ranging,
    Expansion,
    Contraction,
    HighVolatility,
    LowVolatility,
    Transition,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegimeGrade {
    Forbidden,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegimeAssessment {
    pub expectancy: Decimal,
    pub edge: Decimal,
    pub confidence: Decimal,
    pub stability: Decimal,
    pub drawdown: Decimal,
    pub health: Decimal,
}

impl RegimeAssessment {
    pub fn grade(&self) -> RegimeGrade {
        let denominator = if self.drawdown == Decimal::from(0) {
            Decimal::from(1)
        } else {
            self.drawdown
        };
        let mut score =
            (self.expectancy * self.edge * self.confidence * self.stability * self.health)
                / denominator;

        if score < Decimal::from(0) {
            score = Decimal::from(0);
        }

        if score >= Decimal::from(1000) {
            RegimeGrade::Elite
        } else if score >= Decimal::from(500) {
            RegimeGrade::Strong
        } else if score >= Decimal::from(100) {
            RegimeGrade::Normal
        } else if score >= Decimal::from(10) {
            RegimeGrade::Weak
        } else {
            RegimeGrade::Forbidden
        }
    }
}

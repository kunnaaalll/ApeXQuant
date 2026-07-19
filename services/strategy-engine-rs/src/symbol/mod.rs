use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolGrade {
    Forbidden,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolAssessment {
    pub expectancy: Decimal,
    pub stability: Decimal,
    pub drawdown: Decimal,
    pub confidence: Decimal,
    pub sample_count: u32,
}

impl SymbolAssessment {
    pub fn grade(&self) -> SymbolGrade {
        let denominator = if self.drawdown == Decimal::from(0) {
            Decimal::from(1)
        } else {
            self.drawdown
        };
        let mut score = (self.expectancy * self.stability * self.confidence) / denominator;

        // Apply severe penalties
        if self.sample_count < 20 {
            score *= Decimal::new(1, 1); // 0.1
        } else if self.sample_count < 50 {
            score *= Decimal::new(5, 1); // 0.5
        } else if self.sample_count < 100 {
            score *= Decimal::new(8, 1); // 0.8
        }

        if score >= Decimal::from(1000) {
            SymbolGrade::Elite
        } else if score >= Decimal::from(500) {
            SymbolGrade::Strong
        } else if score >= Decimal::from(100) {
            SymbolGrade::Normal
        } else if score >= Decimal::from(10) {
            SymbolGrade::Weak
        } else {
            SymbolGrade::Forbidden
        }
    }
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdequacyGrade {
    Insufficient,
    Weak,
    Adequate,
    Strong,
    InstitutionalGrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleAdequacy {
    pub count: u32,
}

impl SampleAdequacy {
    pub fn new(count: u32) -> Self {
        Self { count }
    }

    pub fn grade(&self) -> AdequacyGrade {
        if self.count >= 1000 {
            AdequacyGrade::InstitutionalGrade
        } else if self.count >= 300 {
            AdequacyGrade::Strong
        } else if self.count >= 100 {
            AdequacyGrade::Adequate
        } else if self.count >= 50 {
            AdequacyGrade::Weak
        } else {
            AdequacyGrade::Insufficient
        }
    }

    pub fn confidence_penalty(&self) -> Decimal {
        match self.grade() {
            AdequacyGrade::InstitutionalGrade => Decimal::new(0, 0), // 0% penalty
            AdequacyGrade::Strong => Decimal::new(10, 2),            // 10% penalty
            AdequacyGrade::Adequate => Decimal::new(30, 2),          // 30% penalty
            AdequacyGrade::Weak => Decimal::new(60, 2),              // 60% penalty
            AdequacyGrade::Insufficient => Decimal::new(90, 2),      // 90% penalty
        }
    }
}

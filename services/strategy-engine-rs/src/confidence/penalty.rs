use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SampleQualityGrade {
    Insufficient,
    Weak,
    Adequate,
    Strong,
    InstitutionalGrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleQuality {
    pub count: u32,
}

impl SampleQuality {
    pub fn new(count: u32) -> Self {
        Self { count }
    }

    pub fn grade(&self) -> SampleQualityGrade {
        if self.count >= 1000 {
            SampleQualityGrade::InstitutionalGrade
        } else if self.count >= 300 {
            SampleQualityGrade::Strong
        } else if self.count >= 100 {
            SampleQualityGrade::Adequate
        } else if self.count >= 50 {
            SampleQualityGrade::Weak
        } else {
            SampleQualityGrade::Insufficient
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfidencePenalty {
    pub amount: Decimal,
}

impl ConfidencePenalty {
    pub fn calculate(drawdown: Decimal, variance: Decimal, instability: Decimal, edge_decay: Decimal, consecutive_losses: u32) -> Self {
        // Penalty = Drawdown + Variance + Instability + Edge Decay + Consecutive Losses Penalty
        let loss_penalty = Decimal::from(consecutive_losses) * Decimal::new(5, 1); // 0.5 per loss
        let total = drawdown + variance + instability + edge_decay + loss_penalty;
        Self { amount: total }
    }
}

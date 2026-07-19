use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningAssessment {
    Strengthening,
    Stable,
    Weakening,
    Collapsing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAccumulator {
    pub recent_wins: u32,
    pub recent_losses: u32,
    pub expectancy_history_ema: Decimal,
    pub confidence_history_ema: Decimal,
    pub edge_history_ema: Decimal,
}

impl EvidenceAccumulator {
    pub fn new() -> Self {
        Self {
            recent_wins: 0,
            recent_losses: 0,
            expectancy_history_ema: Decimal::from(0),
            confidence_history_ema: Decimal::from(0),
            edge_history_ema: Decimal::from(0),
        }
    }
}

impl Default for EvidenceAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceAccumulator {
    pub fn record_event(
        &mut self,
        is_win: bool,
        expectancy: Decimal,
        confidence: Decimal,
        edge: Decimal,
    ) {
        if is_win {
            self.recent_wins += 1;
        } else {
            self.recent_losses += 1;
        }

        let alpha = Decimal::new(1, 1); // 0.1
        let one_minus_alpha = Decimal::new(9, 1); // 0.9

        self.expectancy_history_ema =
            (expectancy * alpha) + (self.expectancy_history_ema * one_minus_alpha);
        self.confidence_history_ema =
            (confidence * alpha) + (self.confidence_history_ema * one_minus_alpha);
        self.edge_history_ema = (edge * alpha) + (self.edge_history_ema * one_minus_alpha);
    }

    pub fn assess(&self) -> LearningAssessment {
        if self.edge_history_ema > Decimal::new(2, 1) {
            // > 0.2
            LearningAssessment::Strengthening
        } else if self.edge_history_ema < Decimal::new(-5, 1) {
            // < -0.5
            LearningAssessment::Collapsing
        } else if self.edge_history_ema < Decimal::from(0) {
            // < 0.0
            LearningAssessment::Weakening
        } else {
            LearningAssessment::Stable
        }
    }
}

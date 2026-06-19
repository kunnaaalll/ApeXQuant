use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone)]
pub struct TailRiskAssessment {
    largest_loss: Decimal,
    sum_tail_loss: Decimal,
    tail_event_count: u64,
}

impl Default for TailRiskAssessment {
    fn default() -> Self {
        Self::new()
    }
}

impl TailRiskAssessment {
    pub fn new() -> Self {
        Self {
            largest_loss: Decimal::ZERO,
            sum_tail_loss: Decimal::ZERO,
            tail_event_count: 0,
        }
    }

    pub fn record_tail_loss(&mut self, loss: Decimal) {
        if loss > Decimal::ZERO {
            if loss > self.largest_loss {
                self.largest_loss = loss;
            }
            self.sum_tail_loss += loss;
            self.tail_event_count += 1;
        }
    }

    pub fn average_tail_loss(&self) -> Decimal {
        if self.tail_event_count == 0 {
            Decimal::ZERO
        } else {
            self.sum_tail_loss / Decimal::from(self.tail_event_count)
        }
    }

    pub fn frequency_of_extreme_events(&self, total_events: u64) -> Decimal {
        if total_events == 0 {
            Decimal::ZERO
        } else {
            Decimal::from(self.tail_event_count) / Decimal::from(total_events)
        }
    }

    /// bounded 0 -> 100
    pub fn tail_risk_score(&self, total_events: u64) -> u32 {
        let avg_loss = self.average_tail_loss();
        // Just a bounded function based on severity and frequency
        let frequency = self.frequency_of_extreme_events(total_events);
        
        let score = (avg_loss * Decimal::new(100, 0)) + (frequency * Decimal::new(500, 0));
        let raw_score = score.to_u32().unwrap_or(0);
        
        if raw_score > 100 {
            100
        } else {
            raw_score
        }
    }

    pub fn tail_severity(&self, total_events: u64) -> TailSeverity {
        let score = self.tail_risk_score(total_events);
        if score >= 90 {
            TailSeverity::Collapse
        } else if score >= 75 {
            TailSeverity::Extreme
        } else if score >= 50 {
            TailSeverity::High
        } else if score >= 25 {
            TailSeverity::Normal
        } else if score >= 10 {
            TailSeverity::Low
        } else {
            TailSeverity::VeryLow
        }
    }

    pub fn get_largest_loss(&self) -> Decimal {
        self.largest_loss
    }
}

// In rust, you cannot add variants to an existing enum inside another file if it wasn't defined.
// The specs state states for Tail Risk: VeryLow, Low, Normal, High, Extreme, Collapse.
// Wait, the main Severity enum has Normal, Elevated, High, Critical, Collapse.
// I should use the same Severity enum or redefine it. Let's redefine Severity for Tail Risk if necessary, or modify `Severity` in `mod.rs`.
// Actually, let's redefine a TailRiskSeverity or modify mod.rs Severity.
// The specs say: "VeryLow, Low, Normal, High, Extreme, Collapse".
// Let's create `TailSeverity` in this file.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TailSeverity {
    VeryLow,
    Low,
    Normal,
    High,
    Extreme,
    Collapse,
}



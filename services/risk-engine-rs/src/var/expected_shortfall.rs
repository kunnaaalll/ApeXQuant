use super::Severity;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ExpectedShortfallAssessment {
    var_threshold: Decimal,
    tail_losses: Vec<Decimal>,
}

impl ExpectedShortfallAssessment {
    pub fn new(var_threshold: Decimal) -> Self {
        Self {
            var_threshold,
            tail_losses: Vec::new(),
        }
    }

    pub fn update_threshold(&mut self, new_var: Decimal) {
        self.var_threshold = new_var;
    }

    pub fn add_loss(&mut self, loss: Decimal) {
        if loss > self.var_threshold {
            self.tail_losses.push(loss);
        }
    }

    /// Measure average loss beyond VaR threshold.
    /// Mathematical invariant: ExpectedShortfall >= VaR
    pub fn compute_shortfall(&self) -> Decimal {
        if self.tail_losses.is_empty() {
            return self.var_threshold; // Invariant ES >= VaR
        }

        let mut sum = Decimal::ZERO;
        for &loss in &self.tail_losses {
            sum += loss;
        }

        let avg_tail_loss = sum / Decimal::from(self.tail_losses.len());

        if avg_tail_loss < self.var_threshold {
            self.var_threshold
        } else {
            avg_tail_loss
        }
    }

    pub fn tail_severity(&self) -> Severity {
        let shortfall = self.compute_shortfall();
        let ratio = if self.var_threshold > Decimal::ZERO {
            shortfall / self.var_threshold
        } else {
            Decimal::new(10, 1) // 1.0
        };

        if ratio >= Decimal::new(30, 1) {
            // 3.0
            Severity::Collapse
        } else if ratio >= Decimal::new(20, 1) {
            // 2.0
            Severity::Critical
        } else if ratio >= Decimal::new(15, 1) {
            // 1.5
            Severity::High
        } else if ratio >= Decimal::new(12, 1) {
            // 1.2
            Severity::Elevated
        } else {
            Severity::Normal
        }
    }
}

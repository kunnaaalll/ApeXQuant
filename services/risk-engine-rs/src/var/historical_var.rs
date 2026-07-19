use super::confidence_levels::ConfidenceLevel;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct HistoricalVaR {
    rolling_losses: Vec<Decimal>, // Tracked losses (positive numbers mean loss, or negative mean return, let's store returns)
    max_window_size: usize,
}

impl HistoricalVaR {
    pub fn new(max_window_size: usize) -> Self {
        Self {
            rolling_losses: Vec::with_capacity(max_window_size),
            max_window_size,
        }
    }

    /// Adds a new return to the rolling window.
    pub fn add_return(&mut self, ret: Decimal) {
        if self.rolling_losses.len() >= self.max_window_size {
            self.rolling_losses.remove(0);
        }
        self.rolling_losses.push(ret);
    }

    /// Computes the historical VaR at a given confidence level.
    /// VaR is strictly >= 0.
    pub fn compute_var(&self, level: ConfidenceLevel) -> Decimal {
        if self.rolling_losses.is_empty() {
            return Decimal::ZERO;
        }

        let mut sorted_returns = self.rolling_losses.clone();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let percentile = level.percentile();
        let index = (Decimal::from(sorted_returns.len()) * percentile).floor();
        let idx = usize::try_from(index.to_i64().unwrap_or(0)).unwrap_or(0);

        let target_idx = if idx > 0 { idx - 1 } else { 0 };
        let worst_return = sorted_returns[target_idx];

        if worst_return < Decimal::ZERO {
            -worst_return
        } else {
            Decimal::ZERO
        }
    }

    /// Returns the worst-case loss (highest absolute negative return)
    pub fn worst_case_loss(&self) -> Decimal {
        let mut max_loss = Decimal::ZERO;
        for &ret in &self.rolling_losses {
            if ret < Decimal::ZERO {
                let loss = -ret;
                if loss > max_loss {
                    max_loss = loss;
                }
            }
        }
        max_loss
    }
}

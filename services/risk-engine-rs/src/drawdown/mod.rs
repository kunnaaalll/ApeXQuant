pub mod events;
pub mod snapshot;
pub mod state;
#[cfg(test)]
pub mod tests;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Tracks current and maximum drawdown as a fraction of peak equity.
/// Updated every time a new equity observation arrives.
///
/// Invariants:
///   - `current_drawdown` ∈ [0, 1] (fraction from peak; 0 = at peak)
///   - `max_drawdown` ≥ `current_drawdown` at all times
///   - No unwrap / expect / panic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawdownTracker {
    pub current_drawdown: Decimal,
    pub max_drawdown: Decimal,
    pub peak_equity: Decimal,
    pub trough_equity: Decimal,
}

impl DrawdownTracker {
    pub fn new() -> Self {
        Self {
            current_drawdown: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            peak_equity: Decimal::ZERO,
            trough_equity: Decimal::ZERO,
        }
    }

    /// Observe a new equity value and update peak / drawdown state.
    pub fn observe(&mut self, equity: Decimal) {
        if equity > self.peak_equity {
            // New peak — drawdown resets to zero
            self.peak_equity = equity;
            self.trough_equity = equity;
            self.current_drawdown = Decimal::ZERO;
        } else {
            self.trough_equity = equity;

            if self.peak_equity > Decimal::ZERO {
                let dd = (self.peak_equity - equity) / self.peak_equity;
                let clamped = dd.max(Decimal::ZERO);
                self.current_drawdown = clamped;
                if clamped > self.max_drawdown {
                    self.max_drawdown = clamped;
                }
            }
        }
    }

    /// True when the strategy has recovered to its previous peak.
    pub fn is_recovered(&self) -> bool {
        self.current_drawdown == Decimal::ZERO
    }
}

impl Default for DrawdownTracker {
    fn default() -> Self {
        Self::new()
    }
}

//! Drawdown management and protection
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use time::OffsetDateTime;

/// Drawdown state machine
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum DrawdownState {
    /// Normal operation - no drawdown concerns
    Normal,
    /// Approaching limits - caution advised
    Warning {
        /// Current drawdown percentage
        pct: Decimal,
    },
    /// Soft limit reached - reduce risk
    SoftLimit {
        /// Current drawdown percentage
        pct: Decimal,
    },
    /// Hard limit reached - stop new positions
    HardLimit,
    /// Recovery mode after significant drawdown
    RecoveryMode,
}

impl DrawdownState {
    /// Whether new positions should be blocked
    pub fn blocks_new_positions(&self) -> bool {
        matches!(self, DrawdownState::HardLimit)
    }

    /// Whether position sizing should be reduced
    pub fn requires_reduction(&self) -> bool {
        matches!(self, DrawdownState::SoftLimit { .. } | DrawdownState::Warning { .. } | DrawdownState::RecoveryMode)
    }

    /// Get the reduction factor for position sizing
    pub fn reduction_factor(&self) -> Decimal {
        match self {
            DrawdownState::Normal => Decimal::ONE,
            DrawdownState::Warning { .. } => Decimal::from_f64(0.75).unwrap_or(Decimal::ONE),
            DrawdownState::SoftLimit { .. } => Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
            DrawdownState::HardLimit => Decimal::ZERO,
            DrawdownState::RecoveryMode => Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            DrawdownState::Normal => "No active drawdown",
            DrawdownState::Warning { .. } => "Drawdown approaching limits",
            DrawdownState::SoftLimit { .. } => "Drawdown soft limit reached",
            DrawdownState::HardLimit => "Drawdown hard limit reached - no new positions",
            DrawdownState::RecoveryMode => "In drawdown recovery mode",
        }
    }
}

impl Default for DrawdownState {
    fn default() -> Self {
        DrawdownState::Normal
    }
}

/// Snapshot of drawdown metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownSnapshot {
    /// Peak equity observed
    pub peak_equity: Decimal,
    /// Current equity
    pub current_equity: Decimal,
    /// Current drawdown amount
    pub drawdown_amount: Decimal,
    /// Current drawdown percentage (0-1)
    pub drawdown_pct: Decimal,
    /// Daily drawdown
    pub daily_drawdown: Decimal,
    /// Weekly drawdown
    pub weekly_drawdown: Decimal,
    /// Monthly drawdown
    pub monthly_drawdown: Decimal,
    /// Maximum drawdown observed
    pub max_drawdown: Decimal,
    /// Time in current drawdown (seconds)
    pub drawdown_duration_sec: u64,
    /// Timestamp
    pub timestamp: OffsetDateTime,
}

/// Drawdown tracking engine
pub struct DrawdownEngine {
    /// Hard drawdown limit (0.1 = 10%)
    hard_limit: Decimal,
    /// Soft drawdown limit (0.05 = 5%)
    soft_limit: Decimal,
    /// Warning threshold (0.03 = 3%)
    warning_threshold: Decimal,
    /// Peak equity seen
    peak_equity: Decimal,
    /// Equity history for time-based drawdowns
    equity_history: VecDeque<(OffsetDateTime, Decimal)>,
    /// When drawdown started
    drawdown_start: Option<OffsetDateTime>,
    /// Maximum drawdown observed
    max_observed_drawdown: Decimal,
    /// Recovery mode flag
    in_recovery: bool,
    /// Recovery target (equity level to exit recovery)
    recovery_target: Decimal,
}

impl DrawdownEngine {
    /// Create new drawdown engine
    pub fn new(hard_limit: Decimal, soft_limit: Decimal) -> Self {
        Self {
            hard_limit,
            soft_limit,
            warning_threshold: soft_limit * Decimal::from_f64(0.6).unwrap_or(Decimal::ONE),
            peak_equity: Decimal::ZERO,
            equity_history: VecDeque::with_capacity(1000),
            drawdown_start: None,
            max_observed_drawdown: Decimal::ZERO,
            in_recovery: false,
            recovery_target: Decimal::ZERO,
        }
    }

    /// Evaluate current drawdown state
    pub fn evaluate(&self, current_equity: Decimal, _balance: Decimal) -> DrawdownState {
        if self.in_recovery {
            if current_equity >= self.recovery_target {
                // Would exit recovery mode here (in real implementation)
                return DrawdownState::Normal;
            }
            return DrawdownState::RecoveryMode;
        }

        if current_equity < self.peak_equity {
            let drawdown_pct = (self.peak_equity - current_equity) / self.peak_equity;

            if drawdown_pct >= self.hard_limit {
                return DrawdownState::HardLimit;
            }

            if drawdown_pct >= self.soft_limit {
                return DrawdownState::SoftLimit { pct: drawdown_pct };
            }

            if drawdown_pct >= self.warning_threshold {
                return DrawdownState::Warning { pct: drawdown_pct };
            }
        }

        DrawdownState::Normal
    }

    /// Update equity and track drawdown
    pub fn update_equity(&mut self, equity: Decimal) {
        let now = OffsetDateTime::now_utc();

        // Track peak
        if equity > self.peak_equity {
            self.peak_equity = equity;
            self.drawdown_start = None;

            // Check recovery
            if self.in_recovery && equity >= self.recovery_target {
                self.in_recovery = false;
            }
        } else if self.drawdown_start.is_none() {
            self.drawdown_start = Some(now);
        }

        // Update history
        self.equity_history.push_back((now, equity));

        // Prune old history (keep 30 days)
        let cutoff = now - time::Duration::days(30);
        while self.equity_history.front().map_or(false, |(t, _)| *t < cutoff) {
            self.equity_history.pop_front();
        }

        // Track max drawdown
        if self.peak_equity > Decimal::ZERO {
            let current_dd = (self.peak_equity - equity) / self.peak_equity;
            if current_dd > self.max_observed_drawdown {
                self.max_observed_drawdown = current_dd;
            }
        }
    }

    /// Enter recovery mode
    pub fn enter_recovery(&mut self, current_equity: Decimal, recovery_pct: Decimal) {
        self.in_recovery = true;
        self.recovery_target = current_equity * (Decimal::ONE + recovery_pct);
    }

    /// Get current snapshot
    pub fn snapshot(&self, current_equity: Decimal) -> DrawdownSnapshot {
        let now = OffsetDateTime::now_utc();
        let drawdown_amount = self.peak_equity.saturating_sub(current_equity);
        let drawdown_pct = if self.peak_equity > Decimal::ZERO {
            drawdown_amount / self.peak_equity
        } else {
            Decimal::ZERO
        };

        DrawdownSnapshot {
            peak_equity: self.peak_equity,
            current_equity,
            drawdown_amount,
            drawdown_pct,
            daily_drawdown: self.calculate_period_drawdown(1),
            weekly_drawdown: self.calculate_period_drawdown(7),
            monthly_drawdown: self.calculate_period_drawdown(30),
            max_drawdown: self.max_observed_drawdown,
            drawdown_duration_sec: self.drawdown_start.map_or(0, |start| {
                (now - start).whole_seconds() as u64
            }),
            timestamp: now,
        }
    }

    fn calculate_period_drawdown(&self, days: i64) -> Decimal {
        let now = OffsetDateTime::now_utc();
        let cutoff = now - time::Duration::days(days);

        let period_equities: Vec<_> = self.equity_history
            .iter()
            .filter(|(t, _)| *t >= cutoff)
            .map(|(_, e)| *e)
            .collect();

        if period_equities.is_empty() {
            return Decimal::ZERO;
        }

        let period_peak = period_equities.iter().copied().max().unwrap_or(Decimal::ZERO);
        let period_low = period_equities.iter().copied().min().unwrap_or(Decimal::ZERO);

        if period_peak > Decimal::ZERO {
            (period_peak - period_low) / period_peak
        } else {
            Decimal::ZERO
        }
    }

    /// Get equity curve slope (positive = growing, negative = declining)
    pub fn equity_curve_slope(&self, days: i64) -> Decimal {
        let now = OffsetDateTime::now_utc();
        let cutoff = now - time::Duration::days(days);

        let points: Vec<_> = self.equity_history
            .iter()
            .filter(|(t, _)| *t >= cutoff)
            .collect();

        if points.len() < 2 {
            return Decimal::ZERO;
        }

        let first = points.first().unwrap().1;
        let last = points.last().unwrap().1;

        (last - first) / first
    }

    /// Reset drawdown tracking (e.g., after withdrawal/deposit)
    pub fn reset(&mut self, new_base_equity: Decimal) {
        self.peak_equity = new_base_equity;
        self.drawdown_start = None;
        self.in_recovery = false;
        self.equity_history.clear();
    }
}

impl Default for DrawdownEngine {
    fn default() -> Self {
        Self::new(
            Decimal::from_f64(0.10).unwrap(), // 10% hard limit
            Decimal::from_f64(0.05).unwrap(), // 5% soft limit
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawdown_state_transitions() {
        let engine = DrawdownEngine::new(
            Decimal::from_f64(0.10).unwrap(),
            Decimal::from_f64(0.05).unwrap(),
        );

        // At peak - normal
        assert_eq!(engine.evaluate(Decimal::from(10000), Decimal::from(10000)), DrawdownState::Normal);
    }

    #[test]
    fn test_drawdown_blocks_positions() {
        assert!(DrawdownState::HardLimit.blocks_new_positions());
        assert!(!DrawdownState::Warning { pct: Decimal::from_f64(0.04).unwrap() }.blocks_new_positions());
    }

    #[test]
    fn test_reduction_factors() {
        assert_eq!(DrawdownState::Normal.reduction_factor(), Decimal::ONE);
        assert!(DrawdownState::SoftLimit { pct: Decimal::ZERO }.reduction_factor() < Decimal::ONE);
        assert_eq!(DrawdownState::HardLimit.reduction_factor(), Decimal::ZERO);
    }

    #[test]
    fn test_peak_tracking() {
        let mut engine = DrawdownEngine::new(
            Decimal::from_f64(0.10).unwrap(),
            Decimal::from_f64(0.05).unwrap(),
        );

        engine.update_equity(Decimal::from(10000));
        assert_eq!(engine.peak_equity, Decimal::from(10000));

        engine.update_equity(Decimal::from(10500));
        assert_eq!(engine.peak_equity, Decimal::from(10500));
    }
}

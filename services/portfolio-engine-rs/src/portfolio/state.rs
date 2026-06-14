use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::errors::PortfolioError;
use super::events::PortfolioEvent;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RecoveryState {
    #[default]
    Normal,
    Recovery,
    Warning,
    Critical,
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioState {
    pub balance: Decimal,
    pub equity: Decimal,
    pub free_margin: Decimal,
    pub used_margin: Decimal,
    pub margin_level: Decimal, // Represented as a percentage ratio, or max if used_margin is 0
    pub floating_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub daily_pnl: Decimal,
    pub weekly_pnl: Decimal,
    pub monthly_pnl: Decimal,
    pub active_positions: usize,
    pub exposure: Decimal,
    pub heat: Decimal,
    pub drawdown: Decimal,
    pub peak_equity: Decimal,
    pub lowest_equity: Decimal,
    pub recovery_state: RecoveryState,
    pub timestamp: OffsetDateTime,
}

impl Default for PortfolioState {
    fn default() -> Self {
        Self {
            balance: Decimal::ZERO,
            equity: Decimal::ZERO,
            free_margin: Decimal::ZERO,
            used_margin: Decimal::ZERO,
            margin_level: Decimal::MAX,
            floating_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            daily_pnl: Decimal::ZERO,
            weekly_pnl: Decimal::ZERO,
            monthly_pnl: Decimal::ZERO,
            active_positions: 0,
            exposure: Decimal::ZERO,
            heat: Decimal::ZERO,
            drawdown: Decimal::ZERO,
            peak_equity: Decimal::ZERO,
            lowest_equity: Decimal::MAX,
            recovery_state: RecoveryState::Normal,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl PortfolioState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Recalculates derived fields (equity, free_margin, margin_level) and 
    /// returns an error if any invariant is violated.
    pub fn validate_invariants(&mut self) -> Result<(), PortfolioError> {
        // Equity = Balance + FloatingPnL
        let expected_equity = self.balance + self.floating_pnl;
        if self.equity != expected_equity {
            return Err(PortfolioError::InvalidEquity {
                equity: self.equity,
                balance: self.balance,
                floating_pnl: self.floating_pnl,
            });
        }

        // FreeMargin = Equity - UsedMargin
        let expected_free_margin = self.equity - self.used_margin;
        if self.free_margin != expected_free_margin {
            return Err(PortfolioError::InvalidFreeMargin {
                free_margin: self.free_margin,
                equity: self.equity,
                used_margin: self.used_margin,
            });
        }

        // MarginLevel > 0
        if self.used_margin.is_zero() {
            self.margin_level = Decimal::MAX;
        } else {
            self.margin_level = self.equity / self.used_margin;
            if self.margin_level.is_sign_negative() {
                return Err(PortfolioError::NegativeMarginLevel {
                    margin_level: self.margin_level,
                });
            }
        }

        // PeakEquity >= Equity
        if self.equity > self.peak_equity {
            self.peak_equity = self.equity;
        }
        if self.peak_equity < self.equity {
            return Err(PortfolioError::PeakEquityLowerThanEquity {
                peak_equity: self.peak_equity,
                equity: self.equity,
            });
        }

        // Update Lowest Equity
        if self.equity < self.lowest_equity {
            self.lowest_equity = self.equity;
        }

        // Drawdown
        // Drawdown calculation depends on peak_equity. Usually (Peak - Equity) / Peak
        // Here we just ensure it's >= 0. Real calculation should be handled specifically if needed.
        if self.drawdown.is_sign_negative() {
            return Err(PortfolioError::NegativeDrawdown {
                drawdown: self.drawdown,
            });
        }

        Ok(())
    }

    pub fn apply_event(&mut self, event: &PortfolioEvent, timestamp: OffsetDateTime) -> Result<(), PortfolioError> {
        self.timestamp = timestamp;

        match event {
            PortfolioEvent::PositionOpened { margin_used, exposure, .. } => {
                self.used_margin += margin_used;
                self.exposure += exposure;
                self.active_positions += 1;
            }
            PortfolioEvent::PositionClosed { realized_pnl, margin_released, exposure_released, .. } => {
                self.balance += realized_pnl;
                self.realized_pnl += realized_pnl;
                self.daily_pnl += realized_pnl;
                self.weekly_pnl += realized_pnl;
                self.monthly_pnl += realized_pnl;
                self.used_margin -= margin_released;
                self.exposure -= exposure_released;
                if self.active_positions == 0 {
                    return Err(PortfolioError::NegativeActivePositions);
                }
                self.active_positions -= 1;
            }
            PortfolioEvent::PartialClose { realized_pnl, margin_released, exposure_released, .. } => {
                self.balance += realized_pnl;
                self.realized_pnl += realized_pnl;
                self.daily_pnl += realized_pnl;
                self.weekly_pnl += realized_pnl;
                self.monthly_pnl += realized_pnl;
                self.used_margin -= margin_released;
                self.exposure -= exposure_released;
            }
            PortfolioEvent::PnlUpdate { pnl_delta, .. } => {
                self.floating_pnl += pnl_delta;
            }
            PortfolioEvent::BalanceChange { amount, .. } => {
                self.balance += amount;
            }
            PortfolioEvent::Deposit { amount } => {
                self.balance += amount;
            }
            PortfolioEvent::Withdrawal { amount } => {
                self.balance -= amount;
            }
            PortfolioEvent::MarginChange { margin_delta, .. } => {
                self.used_margin += margin_delta;
            }
            PortfolioEvent::DrawdownChange { new_drawdown } => {
                self.drawdown = *new_drawdown;
            }
            PortfolioEvent::RecoveryTransition { new_state, .. } => {
                // In a real system, validate valid state machine transitions.
                self.recovery_state = *new_state;
            }
        }

        // Recompute derived fields based on absolute values updated above.
        self.equity = self.balance + self.floating_pnl;
        self.free_margin = self.equity - self.used_margin;

        // Ensure all invariants hold after the event is applied.
        self.validate_invariants()
    }
}

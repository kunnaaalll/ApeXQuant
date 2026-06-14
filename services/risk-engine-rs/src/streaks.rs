//! Losing streak detection and risk adjustment
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::MathematicalOps;

use crate::TradeResult;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// State of trade streaks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreakState {
    /// Number of consecutive losses
    pub consecutive_losses: u32,
    /// Number of consecutive wins
    pub consecutive_wins: u32,
    /// Total trades in sample
    pub total_trades: u32,
    /// Number of wins
    pub wins: u32,
    /// Number of losses
    pub losses: u32,
    /// Win rate
    pub win_rate: Decimal,
}

impl StreakState {
    /// Whether we are in a significant losing streak
    pub fn in_losing_streak(&self) -> bool {
        self.consecutive_losses >= 3
    }

    /// Whether win rate degraded
    pub fn degraded_performance(&self) -> bool {
        self.win_rate < Decimal::from_f64(0.4).unwrap_or(Decimal::ZERO) && self.total_trades >= 10
    }

    /// Get reduction factor based on streak
    pub fn streak_factor(&self) -> Decimal {
        if self.consecutive_losses == 0 {
            return Decimal::ONE;
        }

        // Reduce by 10% per loss, max 50% reduction
        let reduction = Decimal::from(self.consecutive_losses.min(5)) / Decimal::from(10);
        Decimal::ONE - reduction
    }

    /// Get expectancy estimate based on recent performance
    pub fn expectancy(&self, avg_win: Decimal, avg_loss: Decimal) -> Decimal {
        if avg_loss == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let win_prob = self.win_rate;
        let loss_prob = Decimal::ONE - win_prob;

        (win_prob * avg_win) - (loss_prob * avg_loss)
    }
}

impl Default for StreakState {
    fn default() -> Self {
        Self {
            consecutive_losses: 0,
            consecutive_wins: 0,
            total_trades: 0,
            wins: 0,
            losses: 0,
            win_rate: Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
        }
    }
}

/// Streak analysis engine
pub struct StreakAnalyzer {
    /// Minimum trades to consider statistically significant
    min_sample_size: usize,
    /// Consecutive loss threshold for warning
    warning_threshold: u32,
    /// Critical threshold for aggressive reduction
    critical_threshold: u32,
}

impl StreakAnalyzer {
    /// Create new streak analyzer
    pub fn new() -> Self {
        Self {
            min_sample_size: 10,
            warning_threshold: 3,
            critical_threshold: 5,
        }
    }

    /// Analyze recent trades for streak patterns
    pub fn analyze(&self, recent_trades: &[TradeResult]) -> StreakState {
        if recent_trades.is_empty() {
            return StreakState::default();
        }

        let consecutive_losses = self.count_consecutive_losses(recent_trades);
        let consecutive_wins = self.count_consecutive_wins(recent_trades);

        let wins = recent_trades.iter().filter(|t| t.is_win).count() as u32;
        let losses = recent_trades.len() as u32 - wins;

        let win_rate = if !recent_trades.is_empty() {
            Decimal::from(wins) / Decimal::from(recent_trades.len() as u32)
        } else {
            Decimal::from_f64(0.5).unwrap()
        };

        StreakState {
            consecutive_losses,
            consecutive_wins,
            total_trades: recent_trades.len() as u32,
            wins,
            losses,
            win_rate,
        }
    }

    /// Check if trading should be paused
    pub fn should_pause_trading(&self, state: &StreakState) -> bool {
        state.consecutive_losses >= self.critical_threshold
            || (state.degraded_performance() && state.consecutive_losses >= self.warning_threshold)
    }

    /// Get position size adjustment based on streak state
    pub fn get_adjustment(&self, state: &StreakState) -> Decimal {
        if state.consecutive_losses >= self.critical_threshold {
            // Major reduction for critical streak
            return Decimal::from_f64(0.25).unwrap_or(Decimal::ONE);
        }

        if state.consecutive_losses >= self.warning_threshold {
            return Decimal::from_f64(0.5).unwrap_or(Decimal::ONE);
        }

        if state.degraded_performance() {
            return Decimal::from_f64(0.75).unwrap_or(Decimal::ONE);
        }

        Decimal::ONE
    }

    /// Calculate volatility of returns
    pub fn return_volatility(&self, trades: &[TradeResult]) -> Decimal {
        if trades.len() < 2 {
            return Decimal::ZERO;
        }

        let returns: Vec<_> = trades.iter().map(|t| t.pnl).collect();
        self.calculate_std_dev(&returns)
    }

    /// Detect expectancy degradation
    pub fn expectancy_trend(&self, trades: &[TradeResult]) -> ExpectancyTrend {
        if trades.len() < self.min_sample_size {
            return ExpectancyTrend::InsufficientData;
        }

        let third = trades.len() / 3;
        let recent = &trades[trades.len() - third..];
        let previous = &trades[trades.len() - 2 * third..trades.len() - third];

        let recent_expectancy = self.calculate_expectancy(recent);
        let previous_expectancy = self.calculate_expectancy(previous);

        if recent_expectancy < previous_expectancy * Decimal::from_f64(0.5).unwrap_or(Decimal::ONE) {
            ExpectancyTrend::Declining
        } else if recent_expectancy > previous_expectancy * Decimal::from_f64(1.2).unwrap_or(Decimal::ONE) {
            ExpectancyTrend::Improving
        } else {
            ExpectancyTrend::Stable
        }
    }

    fn count_consecutive_losses(&self, trades: &[TradeResult]) -> u32 {
        trades
            .iter()
            .rev()
            .take_while(|t| !t.is_win)
            .count() as u32
    }

    fn count_consecutive_wins(&self, trades: &[TradeResult]) -> u32 {
        trades
            .iter()
            .rev()
            .take_while(|t| t.is_win)
            .count() as u32
    }

    fn calculate_std_dev(&self, values: &[Decimal]) -> Decimal {
        if values.len() < 2 {
            return Decimal::ZERO;
        }

        let mean = values.iter().sum::<Decimal>() / Decimal::from(values.len() as u32);
        let variance: Decimal = values
            .iter()
            .map(|v| {
                let diff = *v - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(values.len() as u32);

        // Approximate sqrt
        let sqrt_approx = variance.sqrt().unwrap_or(Decimal::ZERO);
        sqrt_approx
    }

    fn calculate_expectancy(&self, trades: &[TradeResult]) -> Decimal {
        let wins: Vec<_> = trades.iter().filter(|t| t.is_win).collect();
        let losses: Vec<_> = trades.iter().filter(|t| !t.is_win).collect();

        if losses.is_empty() {
            return Decimal::ZERO;
        }

        let avg_win = if !wins.is_empty() {
            wins.iter().map(|t| t.pnl).sum::<Decimal>() / Decimal::from(wins.len() as u32)
        } else {
            Decimal::ZERO
        };

        let avg_loss = losses.iter().map(|t| t.pnl.abs()).sum::<Decimal>() / Decimal::from(losses.len() as u32);

        let win_rate = Decimal::from(wins.len() as u32) / Decimal::from(trades.len() as u32);

        (win_rate * avg_win) - ((Decimal::ONE - win_rate) * avg_loss)
    }
}

impl Default for StreakAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Trend of expectancy over time
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectancyTrend {
    /// Not enough data
    InsufficientData,
    /// Expectancy improving
    Improving,
    /// Expectancy stable
    Stable,
    /// Expectancy declining
    Declining,
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn make_trade(is_win: bool) -> TradeResult {
        TradeResult {
            pnl: if is_win { Decimal::from(100) } else { Decimal::from(-50) },
            is_win,
            duration_min: 30,
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    fn make_losing_streak(count: usize) -> Vec<TradeResult> {
        (0..count).map(|_| make_trade(false)).collect()
    }

    #[test]
    fn test_consecutive_loss_counting() {
        let analyzer = StreakAnalyzer::new();

        let streak = make_losing_streak(3);
        let state = analyzer.analyze(&streak);

        assert_eq!(state.consecutive_losses, 3);
        assert!(state.in_losing_streak());
    }

    #[test]
    fn test_win_rate_calculation() {
        let analyzer = StreakAnalyzer::new();

        let mut trades = make_losing_streak(5);
        trades.push(make_trade(true));
        trades.push(make_trade(true));
        trades.push(make_trade(true));
        trades.push(make_trade(true));
        trades.push(make_trade(true));

        let state = analyzer.analyze(&trades);
        assert_eq!(state.win_rate, Decimal::from_f64(0.5).unwrap());
    }

    #[test]
    fn test_streak_factor() {
        let state = StreakState {
            consecutive_losses: 3,
            ..Default::default()
        };

        assert_eq!(state.streak_factor(), Decimal::from_f64(0.7).unwrap());
    }

    #[test]
    fn test_should_pause() {
        let analyzer = StreakAnalyzer::new();

        let critical_streak = StreakState {
            consecutive_losses: 5,
            ..Default::default()
        };

        assert!(analyzer.should_pause_trading(&critical_streak));
    }

    #[test]
    fn test_empty_trades_default() {
        let analyzer = StreakAnalyzer::new();
        let state = analyzer.analyze(&[]);

        assert_eq!(state, StreakState::default());
    }
}

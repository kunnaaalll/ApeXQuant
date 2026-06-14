use rust_decimal::prelude::FromPrimitive;
use crate::{PositionSizeResult, RiskInputs, StreakState, TradeResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Kelly Criterion position sizing
///
/// f* = (p * b - q) / b
/// where:
/// - p = probability of win
/// - q = probability of loss (1 - p)
/// - b = average win / average loss (odds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellySizing {
    /// Fraction of Kelly to use (0.25 = quarter Kelly)
    kelly_fraction: Decimal,
    /// Minimum trades required for Kelly calculation
    min_trades: usize,
    /// Minimum win rate to use Kelly (avoid using on bad streaks)
    min_win_rate: Decimal,
}

impl KellySizing {
    /// Create new Kelly sizer
    pub fn new(kelly_fraction: Decimal) -> Self {
        Self {
            kelly_fraction,
            min_trades: 20,
            min_win_rate: Decimal::from_str("0.45").unwrap_or(Decimal::from_f64(0.45).unwrap()),
        }
    }

    /// Calculate Kelly-optimal position size
    pub fn calculate(&self, inputs: &RiskInputs, streak: &StreakState) -> PositionSizeResult {
        // Need sufficient history
        if inputs.recent_trades.len() < self.min_trades {
            return self.fallback_to_fixed(inputs, "Insufficient trade history for Kelly");
        }

        // Avoid Kelly on losing streaks with poor win rate
        if streak.win_rate < self.min_win_rate {
            return self.fallback_to_fixed(inputs, "Win rate too low for Kelly sizing");
        }

        // Calculate win probability and odds
        let (win_prob, avg_win, avg_loss) = self.calculate_statistics(&inputs.recent_trades);

        if avg_loss == Decimal::ZERO {
            return self.fallback_to_fixed(inputs, "No losses in sample");
        }

        let odds = avg_win / avg_loss;
        let loss_prob = Decimal::ONE - win_prob;

        // Kelly formula: f* = (p * b - q) / b
        let kelly_pct = (win_prob * odds - loss_prob) / odds;

        // Apply Kelly fraction for safety
        let adjusted_kelly = kelly_pct * self.kelly_fraction;

        // Clamp to reasonable bounds (0 to max risk percent)
        let max_risk = Decimal::from_str("0.02").unwrap_or(Decimal::ONE);
        let risk_percent = adjusted_kelly.clamp(Decimal::ZERO, max_risk);

        if risk_percent <= Decimal::ZERO {
            return PositionSizeResult {
                lot_size: Decimal::ZERO,
                risk_percent: Decimal::ZERO,
                capital_at_risk: Decimal::ZERO,
                method: super::SizingMethod::KellyFractional,
                reasoning: "Kelly calculation suggests zero or negative expected value".to_string(),
            };
        }

        // Calculate lot size based on stop distance
        let stop_distance = (inputs.entry_price - inputs.stop_loss).abs();
        let capital_at_risk = inputs.equity * risk_percent;
        let lot_size = if stop_distance > Decimal::ZERO {
            capital_at_risk / stop_distance
        } else {
            Decimal::ZERO
        };

        PositionSizeResult {
            lot_size,
            risk_percent,
            capital_at_risk,
            method: super::SizingMethod::KellyFractional,
            reasoning: format!(
                "Kelly sizing: {:.2}% kelly * {:.2}% fraction = {:.2}% risk (win_rate={:.2}, odds={:.2})",
                kelly_pct * Decimal::from(100),
                self.kelly_fraction * Decimal::from(100),
                risk_percent * Decimal::from(100),
                win_prob,
                odds
            ),
        }
    }

    fn calculate_statistics(&self, trades: &[TradeResult]) -> (Decimal, Decimal, Decimal) {
        let wins: Vec<_> = trades.iter().filter(|t| t.is_win).collect();
        let losses: Vec<_> = trades.iter().filter(|t| !t.is_win).collect();

        let win_count = Decimal::from(wins.len() as u64);
        let total_count = Decimal::from(trades.len() as u64);

        let win_prob = if total_count > Decimal::ZERO {
            win_count / total_count
        } else {
            Decimal::ZERO
        };

        let avg_win = if !wins.is_empty() {
            wins.iter().map(|t| t.pnl).sum::<Decimal>() / win_count
        } else {
            Decimal::ZERO
        };

        let avg_loss = if !losses.is_empty() {
            losses.iter().map(|t| t.pnl.abs()).sum::<Decimal>() / Decimal::from(losses.len() as u64)
        } else {
            Decimal::ONE // Avoid division by zero
        };

        (win_prob, avg_win, avg_loss)
    }

    fn fallback_to_fixed(&self, inputs: &RiskInputs, reason: &str) -> PositionSizeResult {
        // Fallback to 0.5% fixed fractional
        let risk_percent = Decimal::from_str("0.005").unwrap_or(Decimal::from_f64(0.005).unwrap());
        let capital_at_risk = inputs.equity * risk_percent;

        let stop_distance = (inputs.entry_price - inputs.stop_loss).abs();
        let lot_size = if stop_distance > Decimal::ZERO {
            (capital_at_risk / stop_distance).max(Decimal::from_str("0.01").unwrap())
        } else {
            Decimal::from_str("0.01").unwrap()
        };

        PositionSizeResult {
            lot_size,
            risk_percent,
            capital_at_risk,
            method: super::SizingMethod::KellyFractional,
            reasoning: format!("{} - using conservative fallback", reason),
        }
    }
}

impl Default for KellySizing {
    fn default() -> Self {
        Self {
            kelly_fraction: Decimal::from_str("0.25").unwrap_or(Decimal::from_f64(0.25).unwrap()),
            min_trades: 20,
            min_win_rate: Decimal::from_str("0.45").unwrap_or(Decimal::from_f64(0.45).unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_test_trades(wins: usize, losses: usize) -> Vec<TradeResult> {
        let mut trades = Vec::new();
        let now = OffsetDateTime::now_utc();

        for _ in 0..wins {
            trades.push(TradeResult {
                pnl: Decimal::from(100),
                is_win: true,
                duration_min: 30,
                timestamp: now,
            });
        }

        for _ in 0..losses {
            trades.push(TradeResult {
                pnl: Decimal::from(-50),
                is_win: false,
                duration_min: 20,
                timestamp: now,
            });
        }

        trades
    }

    fn test_inputs_with_trades(trades: Vec<TradeResult>) -> RiskInputs {
        RiskInputs {
            equity: Decimal::from(10000),
            balance: Decimal::from(10000),
            symbol: "EURUSD".to_string(),
            direction: 1,
            entry_price: Decimal::from_str("1.08500").unwrap(),
            stop_loss: Decimal::from_str("1.08200").unwrap(),
            take_profit: None,
            signal_confidence: Decimal::from_f64(0.8).unwrap(),
            confluence_score: Decimal::from(7),
            regime_quality: Decimal::from_f64(0.7).unwrap(),
            pattern_quality: Decimal::from_f64(0.75).unwrap(),
            atr: None,
            spread: Decimal::from_f64(0.0001).unwrap(),
            open_positions: Vec::new(),
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: trades,
            session: crate::MarketSession::London,
        }
    }

    #[test]
    fn test_kelly_calculation() {
        let sizer = KellySizing::default();
        let trades = create_test_trades(15, 10); // 60% win rate
        let mut inputs = test_inputs_with_trades(trades);

        let streak = StreakState {
            consecutive_losses: 0,
            consecutive_wins: 2,
            total_trades: 25,
            wins: 15,
            losses: 10,
            win_rate: Decimal::from_f64(0.6).unwrap(),
        };

        let result = sizer.calculate(&inputs, &streak);

        assert!(result.lot_size > Decimal::ZERO);
        assert_eq!(result.method, super::SizingMethod::KellyFractional);
    }

    #[test]
    fn test_insufficient_history_fallback() {
        let sizer = KellySizing::default();
        let mut inputs = test_inputs_with_trades(Vec::new());

        let streak = StreakState::default();
        let result = sizer.calculate(&inputs, &streak);

        assert!(result.reasoning.contains("Insufficient trade history"));
    }
}

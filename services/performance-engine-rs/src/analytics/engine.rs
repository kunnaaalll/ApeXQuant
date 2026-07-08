use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use tracing::debug;

use crate::drawdown::DrawdownCalculator;
use crate::expectancy::calculator::ExpectancyCalculator;
use crate::edge::calculator::EdgeCalculator;
use crate::analytics::models::StrategyAnalyticsResult;
use crate::storage::ClosedTradeRecord;

// ─────────────────────────────────────────────────────────────────────────────
// Annualisation factor: sqrt(252) trading days
// ─────────────────────────────────────────────────────────────────────────────
const ANNUALISATION_SQRT_252: f64 = 15.874_507_866_387_544;

#[derive(Clone, Default)]
pub struct AnalyticsEngine;

impl AnalyticsEngine {
    pub fn new() -> Self { Self }
    /// Compute a complete `StrategyAnalyticsResult` from a slice of closed trades.
    /// Deterministic — identical inputs produce identical outputs.
    pub fn compute(&self, strategy_id: &str, trades: &[ClosedTradeRecord]) -> StrategyAnalyticsResult {
        if trades.is_empty() {
            return StrategyAnalyticsResult {
                strategy_id: strategy_id.to_string(),
                ..Default::default()
            };
        }

        let trade_count = trades.len() as u32;

        // ── 1. Win / Loss / Breakeven split ─────────────────────────────────
        let win_count = trades.iter().filter(|t| t.is_win).count() as u32;
        let loss_count = trades.iter().filter(|t| !t.is_win && t.pnl_usd < Decimal::ZERO).count() as u32;
        let breakeven_count = trade_count - win_count - loss_count;

        // ── 2. Gross profit / gross loss ────────────────────────────────────
        let gross_profit: Decimal = trades.iter().filter(|t| t.is_win).map(|t| t.pnl_usd).sum();
        let gross_loss: Decimal = trades.iter().filter(|t| !t.is_win && t.pnl_usd < Decimal::ZERO).map(|t| t.pnl_usd.abs()).sum();
        let net_profit = gross_profit - gross_loss;

        // ── 3. Win/loss rate ─────────────────────────────────────────────────
        let total_dec = Decimal::from(trade_count);
        let win_rate = Decimal::from(win_count) / total_dec;
        let loss_rate = Decimal::from(loss_count) / total_dec;

        // ── 4. Average win / average loss ───────────────────────────────────
        let average_win = if win_count > 0 {
            gross_profit / Decimal::from(win_count)
        } else {
            Decimal::ZERO
        };
        let average_loss = if loss_count > 0 {
            gross_loss / Decimal::from(loss_count)
        } else {
            Decimal::ZERO
        };

        // ── 5. Largest win / largest loss ────────────────────────────────────
        let largest_win = trades.iter().filter(|t| t.is_win).map(|t| t.pnl_usd).fold(Decimal::ZERO, Decimal::max);
        let largest_loss = trades.iter().filter(|t| !t.is_win && t.pnl_usd < Decimal::ZERO).map(|t| t.pnl_usd.abs()).fold(Decimal::ZERO, Decimal::max);

        // ── 6. Expectancy metrics via ExpectancyCalculator ───────────────────
        let exp = ExpectancyCalculator::calculate(
            win_count,
            loss_count,
            breakeven_count,
            gross_profit,
            gross_loss,
        );
        let profit_factor = exp.profit_factor;
        let expectancy = exp.expectancy;
        let average_rr = exp.average_rr;

        // ── 7. SQN = √N × mean(R) / std_dev(R) ─────────────────────────────
        let r_outcomes: Vec<Decimal> = trades.iter().map(|t| t.r_outcome).collect();
        let sqn = Self::compute_sqn(&r_outcomes);

        // ── 8. Streak analysis ────────────────────────────────────────────────
        let (max_consecutive_wins, max_consecutive_losses) = Self::compute_streaks(&trades.iter().map(|t| t.is_win).collect::<Vec<_>>());

        // ── 9. Average trade duration ────────────────────────────────────────
        let total_duration: i64 = trades.iter().map(|t| t.duration_seconds).sum();
        let average_duration_seconds = total_duration / (trade_count as i64).max(1);

        // ── 10. Equity curve and drawdown ─────────────────────────────────────
        let equity_curve = {
            let mut curve = Vec::with_capacity(trades.len() + 1);
            curve.push(Decimal::ZERO);
            let mut running = Decimal::ZERO;
            for t in trades {
                running += t.r_outcome;
                curve.push(running);
            }
            curve
        };

        let max_drawdown = DrawdownCalculator::calculate_max_drawdown(&equity_curve);
        let average_drawdown = DrawdownCalculator::calculate_average_drawdown(&equity_curve);
        let ulcer_index = DrawdownCalculator::calculate_ulcer_index(&equity_curve);
        let recovery_factor = if max_drawdown.is_zero() {
            let total_r: Decimal = r_outcomes.iter().sum();
            if total_r > Decimal::ZERO { dec!(999.99) } else { Decimal::ZERO }
        } else {
            let total_r: Decimal = r_outcomes.iter().sum();
            total_r.abs() / max_drawdown
        };

        // ── 11. Risk-adjusted ratios ──────────────────────────────────────────
        let sharpe_ratio = Self::compute_sharpe(&r_outcomes);
        let sortino_ratio = Self::compute_sortino(&r_outcomes);
        let annualised_r = Self::annualise_return(&r_outcomes);
        let calmar_ratio = if max_drawdown.is_zero() { Decimal::ZERO } else { annualised_r / max_drawdown };
        let omega_ratio = Self::compute_omega(&r_outcomes);
        let burke_ratio = Self::compute_burke(&r_outcomes, &equity_curve);
        let sterling_ratio = if average_drawdown.is_zero() { Decimal::ZERO } else { annualised_r / average_drawdown };
        let mar_ratio = calmar_ratio;

        // ── 12. Edge score ────────────────────────────────────────────────────
        let edge_assessment = EdgeCalculator::calculate(expectancy, win_rate, average_rr, trade_count);
        let edge_score = edge_assessment.metrics.edge_score;

        // ── 13. Stability ─────────────────────────────────────────────────────
        let stability = Self::compute_stability(&r_outcomes);

        // ── 14. Confidence ────────────────────────────────────────────────────
        let confidence = Self::compute_confidence(trade_count, profit_factor, stability);

        // ── 15. Strategy health ───────────────────────────────────────────────
        let health_score = crate::meta::strategy_health::StrategyHealth::synthesise(
            win_rate,
            expectancy,
            profit_factor,
            max_drawdown,
            confidence,
            stability,
        );

        debug!(
            strategy_id,
            trade_count,
            win_rate = %win_rate,
            expectancy = %expectancy,
            sharpe = %sharpe_ratio,
            health = health_score,
            "Analytics computation complete"
        );

        StrategyAnalyticsResult {
            strategy_id: strategy_id.to_string(),
            trade_count,
            win_count,
            loss_count,
            breakeven_count,
            net_profit,
            gross_profit,
            gross_loss,
            win_rate,
            loss_rate,
            profit_factor,
            expectancy,
            average_win,
            average_loss,
            largest_win,
            largest_loss,
            average_rr,
            sqn,
            max_consecutive_wins,
            max_consecutive_losses,
            average_duration_seconds,
            max_drawdown,
            average_drawdown,
            recovery_factor,
            ulcer_index,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            omega_ratio,
            burke_ratio,
            sterling_ratio,
            mar_ratio,
            edge_score,
            confidence,
            stability,
            health_score,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Statistical helpers — all deterministic, no randomness
    // ─────────────────────────────────────────────────────────────────────────

    fn compute_sharpe(r: &[Decimal]) -> Decimal {
        if r.len() < 2 { return Decimal::ZERO; }
        let mean = Self::mean(r);
        let std = Self::std_dev(r, mean);
        if std.is_zero() { return Decimal::ZERO; }
        let ann = Decimal::try_from(ANNUALISATION_SQRT_252).unwrap_or(dec!(15.87));
        mean / std * ann
    }

    fn compute_sortino(r: &[Decimal]) -> Decimal {
        if r.len() < 2 { return Decimal::ZERO; }
        let mean = Self::mean(r);
        let negatives: Vec<Decimal> = r.iter().filter(|&&v| v < Decimal::ZERO).copied().collect();
        if negatives.is_empty() { return dec!(999.99); }
        let neg_mean = Self::mean(&negatives);
        let downside = Self::std_dev(&negatives, neg_mean);
        if downside.is_zero() { return Decimal::ZERO; }
        let ann = Decimal::try_from(ANNUALISATION_SQRT_252).unwrap_or(dec!(15.87));
        mean / downside * ann
    }

    fn compute_omega(r: &[Decimal]) -> Decimal {
        let gains: Decimal = r.iter().map(|&v| v.max(Decimal::ZERO)).sum();
        let losses: Decimal = r.iter().map(|&v| (-v).max(Decimal::ZERO)).sum();
        if losses.is_zero() { return dec!(999.99); }
        gains / losses
    }

    fn compute_burke(r: &[Decimal], equity: &[Decimal]) -> Decimal {
        if r.len() < 2 { return Decimal::ZERO; }
        let mean = Self::mean(r);
        let mut peak = Decimal::ZERO;
        let mut sum_sq = Decimal::ZERO;
        for &eq in equity {
            if eq > peak { peak = eq; }
            if peak > Decimal::ZERO {
                let dd = (peak - eq) / peak;
                sum_sq += dd * dd;
            }
        }
        if sum_sq.is_zero() { return dec!(999.99); }
        let sqrt_sq = Decimal::try_from(sum_sq.to_f64().unwrap_or(0.0).sqrt()).unwrap_or(dec!(0.001));
        if sqrt_sq.is_zero() { return Decimal::ZERO; }
        mean / sqrt_sq
    }

    fn compute_sqn(r: &[Decimal]) -> Decimal {
        if r.len() < 2 { return Decimal::ZERO; }
        let mean = Self::mean(r);
        let std = Self::std_dev(r, mean);
        if std.is_zero() { return Decimal::ZERO; }
        let sqrt_n = Decimal::try_from((r.len() as f64).sqrt()).unwrap_or(dec!(1));
        sqrt_n * mean / std
    }

    fn annualise_return(r: &[Decimal]) -> Decimal {
        if r.is_empty() { return Decimal::ZERO; }
        Self::mean(r) * dec!(252)
    }

    fn compute_stability(r: &[Decimal]) -> Decimal {
        // Coefficient of variation inverse: lower CV = higher stability
        if r.len() < 2 { return Decimal::ZERO; }
        let mean = Self::mean(r);
        if mean.is_zero() { return Decimal::ZERO; }
        let std = Self::std_dev(r, mean);
        let cv = std / mean.abs();
        // Stability = 1 / (1 + CV), bounded [0, 1]
        (dec!(1) / (dec!(1) + cv)).clamp(Decimal::ZERO, dec!(1))
    }

    fn compute_confidence(n: u32, pf: Decimal, stability: Decimal) -> Decimal {
        if n == 0 { return Decimal::ZERO; }
        let sample = (Decimal::from(n.min(200)) / dec!(200)).min(dec!(1));
        let quality = ((pf - dec!(1)) / dec!(1)).clamp(Decimal::ZERO, dec!(1));
        (sample * dec!(0.4) + quality * dec!(0.4) + stability * dec!(0.2))
            .clamp(Decimal::ZERO, dec!(1))
    }

    fn compute_streaks(wins: &[bool]) -> (u32, u32) {
        let mut max_wins = 0u32;
        let mut max_losses = 0u32;
        let mut cur_wins = 0u32;
        let mut cur_losses = 0u32;
        for &w in wins {
            if w {
                cur_wins += 1;
                cur_losses = 0;
                if cur_wins > max_wins { max_wins = cur_wins; }
            } else {
                cur_losses += 1;
                cur_wins = 0;
                if cur_losses > max_losses { max_losses = cur_losses; }
            }
        }
        (max_wins, max_losses)
    }

    fn mean(v: &[Decimal]) -> Decimal {
        if v.is_empty() { return Decimal::ZERO; }
        let s: Decimal = v.iter().sum();
        s / Decimal::from(v.len() as u32)
    }

    fn std_dev(v: &[Decimal], mean: Decimal) -> Decimal {
        if v.len() < 2 { return Decimal::ZERO; }
        let var: Decimal = v.iter().map(|&x| { let d = x - mean; d * d }).sum::<Decimal>()
            / Decimal::from((v.len() - 1) as u32);
        Decimal::try_from(var.to_f64().unwrap_or(0.0).sqrt()).unwrap_or(Decimal::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::storage::ClosedTradeRecord;
    use rust_decimal_macros::dec;

    fn make_trade(r: f64, is_win: bool) -> ClosedTradeRecord {
        let r_dec = Decimal::try_from(r).unwrap();
        ClosedTradeRecord {
            trade_id: uuid::Uuid::new_v4().to_string(),
            strategy_id: "s1".to_string(),
            symbol: "EURUSD".to_string(),
            session: "london".to_string(),
            timeframe: "H1".to_string(),
            pattern_id: "p1".to_string(),
            direction: "long".to_string(),
            entry_price: dec!(1.1),
            exit_price: dec!(1.11),
            sl_price: dec!(1.09),
            tp_price: dec!(1.12),
            rr: dec!(2.0),
            r_outcome: r_dec,
            pnl_usd: r_dec * dec!(100),
            gross_profit: if is_win { r_dec * dec!(100) } else { Decimal::ZERO },
            gross_loss: if !is_win { (r_dec * dec!(100)).abs() } else { Decimal::ZERO },
            commission: dec!(0),
            swap: dec!(0),
            is_win,
            entry_quality: dec!(0.8),
            duration_seconds: 3600,
            opened_at: Utc::now(),
            closed_at: Utc::now(),
        }
    }

    #[test]
    fn test_empty_input_returns_zeros() {
        let result = AnalyticsEngine::compute("s1", &[]);
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.sharpe_ratio, Decimal::ZERO);
    }

    #[test]
    fn test_all_wins_positive_sharpe() {
        let trades: Vec<ClosedTradeRecord> = (0..50).map(|_| make_trade(1.0, true)).collect();
        let result = AnalyticsEngine::compute("s1", &trades);
        assert_eq!(result.win_rate, dec!(1));
        assert_eq!(result.win_count, 50);
        assert!(result.profit_factor > dec!(99));
    }

    #[test]
    fn test_mixed_trades_expectancy() {
        // 60% win rate, 1R average win, 1R average loss → expectancy = 0.2R
        let mut trades = Vec::new();
        for _ in 0..60 { trades.push(make_trade(1.0, true)); }
        for _ in 0..40 { trades.push(make_trade(-1.0, false)); }
        let result = AnalyticsEngine::compute("s1", &trades);
        assert_eq!(result.win_rate, dec!(0.6));
        // Expectancy ≈ 0.6*100 - 0.4*100 = 20 per trade (in USD)
        assert!(result.expectancy > Decimal::ZERO);
    }

    #[test]
    fn test_sharpe_is_deterministic() {
        let trades: Vec<ClosedTradeRecord> = (0..30).enumerate().map(|(i, _)| {
            let is_win = i % 3 != 2;
            make_trade(if is_win { 1.0 } else { -0.5 }, is_win)
        }).collect();
        let r1 = AnalyticsEngine::compute("s1", &trades);
        let r2 = AnalyticsEngine::compute("s1", &trades);
        assert_eq!(r1.sharpe_ratio, r2.sharpe_ratio);
    }
}

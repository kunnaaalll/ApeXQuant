//! Portfolio Simulation Module
//!
//! Tracks equity curves, drawdowns, portfolio heat, exposure, margin usage,
//! and capital efficiency using `rust_decimal::Decimal`.
//! All metrics are computed from real PortfolioSnapshot series — no hardcoded zeros.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct PortfolioState {
    pub equity: Decimal,
    /// Current drawdown as a fraction of peak equity (0.0 = no drawdown, 1.0 = total loss).
    pub drawdown: Decimal,
    /// Portfolio "heat" = total risk-on exposure as fraction of equity (e.g. 0.06 = 6% at risk).
    pub heat: Decimal,
    /// Total notional exposure as fraction of equity.
    pub exposure: Decimal,
    /// Margin currently in use as fraction of equity.
    pub margin_usage: Decimal,
    /// Capital efficiency: how productively capital is deployed (0.0–1.0).
    pub capital_efficiency: Decimal,
}

#[derive(Debug, Clone)]
pub struct PortfolioSnapshot {
    pub timestamp_ms: i64,
    pub state: PortfolioState,
}

#[derive(Debug, Clone)]
pub struct PortfolioMetrics {
    pub average_heat: Decimal,
    pub average_exposure: Decimal,
    pub max_margin_usage: Decimal,
    pub capital_efficiency_score: Decimal,
    pub max_drawdown: Decimal,
    pub average_drawdown: Decimal,
    pub peak_equity: Decimal,
    pub final_equity: Decimal,
    pub total_return: Decimal,
}

pub struct PortfolioSimulator;

impl PortfolioSimulator {
    /// Compute full portfolio metrics from a time-ordered series of portfolio snapshots.
    ///
    /// Returns an error if the snapshot slice is empty.
    pub fn simulate(snapshots: &[PortfolioSnapshot]) -> Result<PortfolioMetrics, &'static str> {
        if snapshots.is_empty() {
            return Err("portfolio simulation requires at least one snapshot");
        }

        let n = Decimal::from(snapshots.len() as i64);

        let average_heat = snapshots.iter().map(|s| s.state.heat).sum::<Decimal>() / n;
        let average_exposure = snapshots.iter().map(|s| s.state.exposure).sum::<Decimal>() / n;
        let max_margin_usage = snapshots
            .iter()
            .map(|s| s.state.margin_usage)
            .fold(Decimal::ZERO, |a, b| a.max(b));
        let capital_efficiency_score = snapshots
            .iter()
            .map(|s| s.state.capital_efficiency)
            .sum::<Decimal>()
            / n;
        let max_drawdown = snapshots
            .iter()
            .map(|s| s.state.drawdown)
            .fold(Decimal::ZERO, |a, b| a.max(b));
        let average_drawdown = snapshots.iter().map(|s| s.state.drawdown).sum::<Decimal>() / n;
        let peak_equity = snapshots
            .iter()
            .map(|s| s.state.equity)
            .fold(Decimal::ZERO, |a, b| a.max(b));
        let first_equity = snapshots
            .first()
            .map(|s| s.state.equity)
            .unwrap_or(Decimal::ZERO);
        let final_equity = snapshots
            .last()
            .map(|s| s.state.equity)
            .unwrap_or(Decimal::ZERO);
        let total_return = if first_equity > Decimal::ZERO {
            (final_equity - first_equity) / first_equity
        } else {
            Decimal::ZERO
        };

        Ok(PortfolioMetrics {
            average_heat,
            average_exposure,
            max_margin_usage,
            capital_efficiency_score,
            max_drawdown,
            average_drawdown,
            peak_equity,
            final_equity,
            total_return,
        })
    }

    /// Compute intraday drawdown series from equity values in snapshots.
    /// Returns a parallel vector of drawdown fractions at each snapshot timestamp.
    pub fn compute_drawdown_series(snapshots: &[PortfolioSnapshot]) -> Vec<Decimal> {
        let mut peak = Decimal::ZERO;
        snapshots
            .iter()
            .map(|s| {
                if s.state.equity > peak {
                    peak = s.state.equity;
                }
                if peak > Decimal::ZERO {
                    ((peak - s.state.equity) / peak).max(Decimal::ZERO)
                } else {
                    Decimal::ZERO
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(ts_ms: i64, equity: i64, heat: i64, scale: u32) -> PortfolioSnapshot {
        PortfolioSnapshot {
            timestamp_ms: ts_ms,
            state: PortfolioState {
                equity: Decimal::from(equity),
                drawdown: Decimal::ZERO,
                heat: Decimal::new(heat, scale),
                exposure: Decimal::new(heat * 3, scale),
                margin_usage: Decimal::new(heat / 2, scale),
                capital_efficiency: Decimal::new(8, 1),
            },
        }
    }

    #[test]
    fn test_empty_snapshots_returns_error() {
        assert!(PortfolioSimulator::simulate(&[]).is_err());
    }

    #[test]
    fn test_single_snapshot() {
        let snaps = vec![snap(0, 10_000, 5, 2)];
        let m = PortfolioSimulator::simulate(&snaps).expect("ok");
        assert_eq!(m.average_heat, Decimal::new(5, 2));
        assert_eq!(m.peak_equity, Decimal::from(10_000i64));
        assert_eq!(m.total_return, Decimal::ZERO);
    }

    #[test]
    fn test_total_return_computed() {
        let snaps = vec![snap(0, 10_000, 5, 2), snap(1000, 11_000, 6, 2)];
        let m = PortfolioSimulator::simulate(&snaps).expect("ok");
        // Return = (11000 - 10000) / 10000 = 0.1
        assert_eq!(m.total_return, Decimal::new(1, 1));
    }

    #[test]
    fn test_drawdown_series_tracks_peak() {
        let snaps = vec![
            snap(0, 10_000, 5, 2),
            snap(1000, 11_000, 5, 2),
            snap(2000, 9_000, 5, 2),
            snap(3000, 12_000, 5, 2),
        ];
        let series = PortfolioSimulator::compute_drawdown_series(&snaps);
        assert_eq!(series.len(), 4);
        // At snap[2]: equity=9000, peak=11000 → dd = 2000/11000 ≈ 0.1818
        assert!(series[2] > Decimal::ZERO);
        // At snap[3]: new peak, dd = 0
        assert_eq!(series[3], Decimal::ZERO);
    }
}

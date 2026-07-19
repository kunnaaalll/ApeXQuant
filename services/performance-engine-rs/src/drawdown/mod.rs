use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

pub struct DrawdownCalculator;

impl DrawdownCalculator {
    /// Calculate maximum drawdown from equity curve
    pub fn calculate_max_drawdown(equity_curve: &[Decimal]) -> Decimal {
        if equity_curve.is_empty() {
            return Decimal::ZERO;
        }

        let mut peak = equity_curve[0];
        let mut max_dd = Decimal::ZERO;

        for &equity in equity_curve {
            if equity > peak {
                peak = equity;
            } else if peak > Decimal::ZERO {
                let dd = (peak - equity) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }
        }

        max_dd
    }

    /// Calculate average drawdown from equity curve
    pub fn calculate_average_drawdown(equity_curve: &[Decimal]) -> Decimal {
        if equity_curve.is_empty() {
            return Decimal::ZERO;
        }

        let mut peak = equity_curve[0];
        let mut drawdowns = Vec::new();

        for &equity in equity_curve {
            if equity > peak {
                peak = equity;
            } else if peak > Decimal::ZERO {
                let dd = (peak - equity) / peak;
                drawdowns.push(dd);
            }
        }

        if drawdowns.is_empty() {
            return Decimal::ZERO;
        }

        let sum: Decimal = drawdowns.iter().sum();
        sum / Decimal::from(drawdowns.len() as u32)
    }

    /// Calculate Ulcer Index (sqrt of average squared drawdowns)
    pub fn calculate_ulcer_index(equity_curve: &[Decimal]) -> Decimal {
        if equity_curve.is_empty() {
            return Decimal::ZERO;
        }

        let mut peak = equity_curve[0];
        let mut sum_sq = Decimal::ZERO;
        let mut n = 0;

        for &equity in equity_curve {
            n += 1;
            if equity > peak {
                peak = equity;
            } else if peak > Decimal::ZERO {
                let dd = (peak - equity) / peak;
                sum_sq += dd * dd;
            }
        }

        if n == 0 {
            return Decimal::ZERO;
        }

        let mean_sq = sum_sq / Decimal::from(n);
        Decimal::try_from(mean_sq.to_f64().unwrap_or(0.0).sqrt()).unwrap_or(Decimal::ZERO)
    }
}

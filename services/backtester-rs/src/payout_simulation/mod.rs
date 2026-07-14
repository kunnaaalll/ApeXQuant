//! Payout Simulation Module
//!
//! Models profit withdrawals, profit splits, and retained capital.
//! All computations use real equity and split configuration — no hardcoded zeros.

use rust_decimal::Decimal;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum PayoutError {
    #[error("starting balance must be positive, got {0}")]
    NonPositiveStartingBalance(Decimal),
    #[error("profit split fractions must sum to 1.0, got {0}")]
    InvalidSplitFractions(Decimal),
    #[error("no profit available: current balance {0} ≤ starting balance {1}")]
    NoProfitAvailable(Decimal, Decimal),
}

/// Eligibility status for a payout request.
#[derive(Debug, Clone)]
pub struct PayoutEligibility {
    pub is_eligible: bool,
    /// Unix milliseconds of the earliest eligible payout date.
    pub next_payout_date_ms: i64,
    /// Minimum balance required above starting balance to qualify.
    pub minimum_balance_required: Decimal,
    /// Reason if not eligible.
    pub ineligibility_reason: Option<String>,
}

/// Profit split configuration.
#[derive(Debug, Clone)]
pub struct ProfitSplit {
    /// Fraction of profit paid to the trader (e.g. 0.80 = 80%).
    pub trader_pct: Decimal,
    /// Fraction of profit retained by the firm (e.g. 0.20 = 20%).
    pub firm_pct: Decimal,
}

impl ProfitSplit {
    /// Standard 80/20 split.
    pub fn standard_80_20() -> Self {
        Self {
            trader_pct: Decimal::new(80, 2),
            firm_pct: Decimal::new(20, 2),
        }
    }

    /// Premium 90/10 split.
    pub fn premium_90_10() -> Self {
        Self {
            trader_pct: Decimal::new(90, 2),
            firm_pct: Decimal::new(10, 2),
        }
    }

    /// Validate that fractions sum to 1.0.
    pub fn validate(&self) -> Result<(), PayoutError> {
        let sum = self.trader_pct + self.firm_pct;
        if (sum - Decimal::ONE).abs() > Decimal::new(1, 4) {
            return Err(PayoutError::InvalidSplitFractions(sum));
        }
        Ok(())
    }
}

/// Payout result for a single payout event.
#[derive(Debug, Clone)]
pub struct PayoutMetrics {
    /// Total profit available (current_balance - starting_balance).
    pub total_profit: Decimal,
    /// Trader's share of the profit.
    pub gross_payout: Decimal,
    /// Firm's share of the profit.
    pub firm_profit: Decimal,
    /// Net payout to the trader (same as gross — tax is caller's responsibility).
    pub net_payout: Decimal,
    /// Balance remaining after payout (starting_balance + firm's share retained).
    pub retained_capital: Decimal,
    /// Timestamp of payout simulation.
    pub timestamp_ms: i64,
}

/// Monthly cash flow summary.
#[derive(Debug, Clone)]
pub struct MonthlyCashFlow {
    pub month: u8,
    pub year: i32,
    pub starting_balance: Decimal,
    pub ending_balance: Decimal,
    pub payout: Decimal,
    pub firm_retained: Decimal,
}

pub struct PayoutSimulator;

impl PayoutSimulator {
    /// Simulate a single payout event.
    ///
    /// After payout, the account is reset to `starting_balance` plus the firm's retained
    /// portion of profits. This follows the standard prop firm payout model where the
    /// trader receives their split and the account is partially reset.
    pub fn simulate_payout(
        current_balance: Decimal,
        starting_balance: Decimal,
        split: &ProfitSplit,
    ) -> Result<PayoutMetrics, PayoutError> {
        if starting_balance <= Decimal::ZERO {
            return Err(PayoutError::NonPositiveStartingBalance(starting_balance));
        }
        split.validate()?;

        if current_balance <= starting_balance {
            return Err(PayoutError::NoProfitAvailable(
                current_balance,
                starting_balance,
            ));
        }

        let total_profit = current_balance - starting_balance;
        let gross_payout = total_profit * split.trader_pct;
        let firm_profit = total_profit * split.firm_pct;
        let net_payout = gross_payout; // Tax calculation is caller's responsibility
        let retained_capital = starting_balance + firm_profit;
        let timestamp_ms = OffsetDateTime::now_utc().unix_timestamp() * 1000;

        Ok(PayoutMetrics {
            total_profit,
            gross_payout,
            firm_profit,
            net_payout,
            retained_capital,
            timestamp_ms,
        })
    }

    /// Check whether a payout is currently eligible.
    ///
    /// Eligibility requires:
    /// 1. Current balance > starting balance (profit exists).
    /// 2. Minimum 14 calendar days since account inception (cooldown period).
    pub fn check_eligibility(
        current_balance: Decimal,
        starting_balance: Decimal,
        account_inception_ms: i64,
        min_profit_threshold: Decimal,
    ) -> PayoutEligibility {
        let now_ms = OffsetDateTime::now_utc().unix_timestamp() * 1000;
        let fourteen_days_ms: i64 = 14 * 24 * 60 * 60 * 1000;
        let eligible_after_ms = account_inception_ms + fourteen_days_ms;

        if current_balance <= starting_balance {
            return PayoutEligibility {
                is_eligible: false,
                next_payout_date_ms: eligible_after_ms,
                minimum_balance_required: starting_balance + min_profit_threshold,
                ineligibility_reason: Some("No profit above starting balance".to_string()),
            };
        }

        let profit = current_balance - starting_balance;
        if profit < min_profit_threshold {
            return PayoutEligibility {
                is_eligible: false,
                next_payout_date_ms: eligible_after_ms,
                minimum_balance_required: starting_balance + min_profit_threshold,
                ineligibility_reason: Some(format!(
                    "Profit {profit} below minimum threshold {min_profit_threshold}"
                )),
            };
        }

        if now_ms < eligible_after_ms {
            return PayoutEligibility {
                is_eligible: false,
                next_payout_date_ms: eligible_after_ms,
                minimum_balance_required: starting_balance + min_profit_threshold,
                ineligibility_reason: Some("Cooldown period not yet elapsed".to_string()),
            };
        }

        PayoutEligibility {
            is_eligible: true,
            next_payout_date_ms: now_ms,
            minimum_balance_required: starting_balance + min_profit_threshold,
            ineligibility_reason: None,
        }
    }

    /// Project monthly cash flow over N months given an assumed monthly return rate.
    pub fn project_monthly_cashflow(
        starting_balance: Decimal,
        monthly_return_fraction: Decimal,
        split: &ProfitSplit,
        months: u32,
    ) -> Result<Vec<MonthlyCashFlow>, PayoutError> {
        split.validate()?;
        let now = OffsetDateTime::now_utc();
        let mut balance = starting_balance;
        let mut result = Vec::with_capacity(months as usize);

        for m in 0..months {
            let start = balance;
            let gross_profit = start * monthly_return_fraction;
            let payout = gross_profit * split.trader_pct;
            let firm_retained = gross_profit * split.firm_pct;
            balance = start + firm_retained; // Reset: starting + firm's retained portion

            result.push(MonthlyCashFlow {
                month: ((now.month() as u8 + m as u8).wrapping_sub(1) % 12) + 1,
                year: now.year() + ((now.month() as u8 + m as u8 - 1) / 12) as i32,
                starting_balance: start,
                ending_balance: balance,
                payout,
                firm_retained,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payout_80_20_split() {
        let split = ProfitSplit::standard_80_20();
        let result = PayoutSimulator::simulate_payout(
            Decimal::from(11_000i64),
            Decimal::from(10_000i64),
            &split,
        )
        .expect("ok");
        assert_eq!(result.total_profit, Decimal::from(1_000i64));
        assert_eq!(result.gross_payout, Decimal::from(800i64));
        assert_eq!(result.firm_profit, Decimal::from(200i64));
        assert_eq!(result.retained_capital, Decimal::from(10_200i64));
    }

    #[test]
    fn test_no_profit_returns_error() {
        let split = ProfitSplit::standard_80_20();
        let result = PayoutSimulator::simulate_payout(
            Decimal::from(10_000i64),
            Decimal::from(10_000i64),
            &split,
        );
        assert!(matches!(result, Err(PayoutError::NoProfitAvailable(_, _))));
    }

    #[test]
    fn test_invalid_split_returns_error() {
        let bad_split = ProfitSplit {
            trader_pct: Decimal::new(90, 2),
            firm_pct: Decimal::new(20, 2),
        };
        let result = PayoutSimulator::simulate_payout(
            Decimal::from(11_000i64),
            Decimal::from(10_000i64),
            &bad_split,
        );
        assert!(matches!(result, Err(PayoutError::InvalidSplitFractions(_))));
    }

    #[test]
    fn test_monthly_projection_grows_balance() {
        let split = ProfitSplit::standard_80_20();
        let flows = PayoutSimulator::project_monthly_cashflow(
            Decimal::from(10_000i64),
            Decimal::new(5, 2), // 5% monthly return
            &split,
            6,
        )
        .expect("ok");
        assert_eq!(flows.len(), 6);
        // Each month should pay out something
        for f in &flows {
            assert!(f.payout > Decimal::ZERO);
        }
    }
}

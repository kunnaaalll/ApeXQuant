//! Payout Simulation Module
//!
//! Models profit withdrawals, payout schedules, firm profit splits, and account scaling.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct PayoutEligibility {
    pub is_eligible: bool,
    pub next_payout_date_ms: i64,
    pub minimum_balance_required: Decimal,
}

#[derive(Debug, Clone)]
pub struct ProfitSplit {
    pub trader_pct: Decimal,
    pub firm_pct: Decimal,
}

#[derive(Debug, Clone)]
pub struct PayoutMetrics {
    pub gross_payout: Decimal,
    pub net_payout: Decimal,
    pub retained_capital: Decimal,
}

pub struct PayoutSimulator;

impl PayoutSimulator {
    pub fn simulate_payout(
        _current_balance: Decimal,
        _starting_balance: Decimal,
        _split: &ProfitSplit,
    ) -> Result<PayoutMetrics, &'static str> {
        // Stub: Calculate payout based on profits and splits
        Ok(PayoutMetrics {
            gross_payout: Decimal::ZERO,
            net_payout: Decimal::ZERO,
            retained_capital: _current_balance,
        })
    }
}

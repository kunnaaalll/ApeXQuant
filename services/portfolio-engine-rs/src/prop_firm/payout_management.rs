use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRequest {
    pub account_id: String,
    pub requested_amount: Decimal,
    pub account_profit: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResult {
    pub approved: bool,
    pub amount: Decimal,
    pub trader_share: Decimal,
    pub firm_share: Decimal,
    pub reason: String,
}

pub struct PayoutEngine;

impl Default for PayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PayoutEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process_request(
        &self,
        request: &PayoutRequest,
        trader_split_percentage: Decimal,
        min_payout_amount: Decimal,
        buffer_required: Decimal,
    ) -> PayoutResult {
        if request.requested_amount < min_payout_amount {
            return PayoutResult {
                approved: false,
                amount: Decimal::ZERO,
                trader_share: Decimal::ZERO,
                firm_share: Decimal::ZERO,
                reason: format!("Requested amount {} is less than minimum {}", request.requested_amount, min_payout_amount),
            };
        }

        let max_available = request.account_profit - buffer_required;

        if request.requested_amount > max_available {
            return PayoutResult {
                approved: false,
                amount: Decimal::ZERO,
                trader_share: Decimal::ZERO,
                firm_share: Decimal::ZERO,
                reason: format!("Requested amount {} exceeds available profit after buffer {}", request.requested_amount, max_available),
            };
        }

        let trader_share = request.requested_amount * trader_split_percentage;
        let firm_share = request.requested_amount - trader_share;

        PayoutResult {
            approved: true,
            amount: request.requested_amount,
            trader_share,
            firm_share,
            reason: "Approved".to_string(),
        }
    }
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::circuit_breaker::ExecutionProtectionState;
use super::severity::Severity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionRiskEvent {
    SpreadChanged {
        symbol: String,
        current_spread: Decimal,
        average_spread: Decimal,
        spread_multiplier: Decimal,
        spread_score: u32,
    },
    LatencyChanged {
        broker_latency_ms: u32,
        exchange_latency_ms: u32,
        network_latency_ms: u32,
        latency_score: u32,
    },
    FailureRecorded {
        error_type: String,
        failure_score: u32,
    },
    RejectionRecorded {
        consecutive_rejections: u32,
        rolling_rejection_rate: Decimal,
    },
    StateTransition {
        from: ExecutionProtectionState,
        to: ExecutionProtectionState,
        reason: String,
    },
    CooldownEvent {
        stable_cycles_completed: u32,
        successful_fills_completed: u32,
    },
    RecoveryEvent {
        from: String,
        to: String,
    },
    AnomalyDetected {
        anomaly_type: String,
        severity: Severity,
    },
    SlippageRecorded {
        expected_slippage: Decimal,
        realized_slippage: Decimal,
        penalty_score: u32,
    },
    LiquidityChanged {
        book_depth: Decimal,
        spread_quality: Decimal,
        regime: String,
    },
    FillQualityRecorded {
        fill_ratio: Decimal,
        grade: String,
    },
}

//! Strategy Sandbox Module
//!
//! Provides isolated environments for testing and evaluating strategies before shadow mode.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub total_trades: usize,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
    pub total_return: Decimal,
    pub is_deterministic: bool,
}

#[derive(Debug, Clone)]
pub struct SandboxSession {
    pub session_id: String,
    pub strategy_id: String,
    pub start_time_ms: i64,
    pub end_time_ms: i64,
}

impl SandboxSession {
    pub fn new(session_id: String, strategy_id: String, start_time_ms: i64, end_time_ms: i64) -> Self {
        Self {
            session_id,
            strategy_id,
            start_time_ms,
            end_time_ms,
        }
    }
}

pub struct StrategySandbox;

impl StrategySandbox {
    pub fn run_session(_session: &SandboxSession) -> Result<SandboxResult, &'static str> {
        // Deterministic isolated replay simulation stub
        Ok(SandboxResult {
            total_trades: 0,
            win_rate: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            total_return: Decimal::ZERO,
            is_deterministic: true,
        })
    }
}

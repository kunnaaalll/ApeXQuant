//! Event Bus Module
//!
//! Stubs/Traits for integration with market-data-engine-rs, strategy-engine-rs,
//! risk-engine-rs, portfolio-engine-rs, execution-engine-rs, learning-engine-rs.

use crate::market_replay::models::ReplayEvent;
use rust_decimal::Decimal;

// Phase 5 Events & Snapshots
#[derive(Debug, Clone)]
pub struct CertificationEvent {
    pub strategy_id: String,
    pub stage: String,
}

#[derive(Debug, Clone)]
pub struct DriftEvent {
    pub strategy_id: String,
    pub drift_severity: String,
}

#[derive(Debug, Clone)]
pub struct ParityEvent {
    pub strategy_id: String,
    pub parity_score: Decimal,
}

#[derive(Debug, Clone)]
pub struct ProductionValidationEvent {
    pub strategy_id: String,
    pub is_ready: bool,
}

#[derive(Debug, Clone)]
pub struct CertificationSnapshot {
    pub strategy_id: String,
    pub metrics: String,
}

#[derive(Debug, Clone)]
pub struct DriftSnapshot {
    pub strategy_id: String,
    pub metrics: String,
}

#[derive(Debug, Clone)]
pub struct BenchmarkSnapshot {
    pub strategy_id: String,
    pub metrics: String,
}

pub trait EventPublisher {
    fn publish_market_event(&mut self, event: ReplayEvent) -> Result<(), &'static str>;
    fn publish_execution_event(&mut self, order_id: &str, status: &str)
        -> Result<(), &'static str>;

    // Phase 3 Learning Engine integration
    fn publish_learning_discovery(&mut self, data: &str) -> Result<(), &'static str>;
    fn publish_learning_optimization(&mut self, data: &str) -> Result<(), &'static str>;
    fn publish_learning_promotion_candidate(
        &mut self,
        strategy_id: &str,
    ) -> Result<(), &'static str>;
    fn publish_learning_retirement_candidate(
        &mut self,
        strategy_id: &str,
    ) -> Result<(), &'static str>;

    // Phase 4 Learning Engine integration
    fn publish_learning_account_optimization(
        &mut self,
        account_id: &str,
    ) -> Result<(), &'static str>;
    fn publish_learning_capital_rotation(&mut self, account_id: &str) -> Result<(), &'static str>;
    fn publish_learning_payout_analysis(&mut self, account_id: &str) -> Result<(), &'static str>;

    // Phase 5 Learning Engine Integration
    fn publish_learning_parity_drift(&mut self, data: &str) -> Result<(), &'static str>;
    fn publish_learning_production_candidate(
        &mut self,
        strategy_id: &str,
    ) -> Result<(), &'static str>;
    fn publish_learning_certification_update(&mut self, data: &str) -> Result<(), &'static str>;
}

pub trait EventSubscriber {
    fn on_market_event(&mut self, event: ReplayEvent);
    fn on_execution_event(&mut self, order_id: &str, status: &str);

    // Phase 3 & 5 Strategy Engine integration
    fn on_strategy_signal(&mut self, signal: &str);
    fn on_strategy_health(&mut self, health_status: &str);
    fn on_strategy_regime(&mut self, regime: &str);
    fn on_strategy_performance(&mut self, data: &str);
    fn on_strategy_lifecycle(&mut self, data: &str);

    // Phase 4 & 5 Portfolio Engine integration
    fn on_portfolio_exposure(&mut self, exposure_data: &str);
    fn on_portfolio_heat(&mut self, heat_data: &str);
    fn on_portfolio_health(&mut self, health_data: &str);
    fn on_portfolio_drawdown(&mut self, data: &str);

    // Phase 5 Execution Engine Integration
    fn on_execution_slippage(&mut self, data: &str);
    fn on_execution_latency(&mut self, data: &str);
    fn on_execution_fill_quality(&mut self, data: &str);

    // Phase 5 Risk Engine Integration
    fn on_risk_interventions(&mut self, data: &str);
    fn on_risk_freeze_events(&mut self, data: &str);
    fn on_risk_circuit_breakers(&mut self, data: &str);
}

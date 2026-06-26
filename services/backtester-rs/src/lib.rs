#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod attribution;
pub mod event_bus;
pub mod execution_model;
pub mod market_replay;
pub mod monte_carlo;
pub mod overfitting;
pub mod parameter_optimizer;
pub mod parity;
pub mod performance;
pub mod portfolio_simulation;
pub mod promotion;
pub mod ranking;
pub mod regime_validation;
pub mod reporting;
pub mod risk_simulation;
pub mod robustness;
pub mod simulation;
pub mod storage;
pub mod strategy_sandbox;
pub mod strategy_simulation;
pub mod walk_forward;

// Phase 4 additions
pub mod account_allocator;
pub mod capital_rotation;
pub mod correlation_simulation;
pub mod funded_account_manager;
pub mod multi_account;
pub mod payout_simulation;
pub mod portfolio_stress;
pub mod prop_constraints;

// Phase 5 additions
pub mod shadow_validation;
pub mod drift_detection;
pub mod certification;
pub mod benchmark;
pub mod production_validator;
pub mod institutional_reporting;

// Phase 6 additions
pub mod adversarial_testing;
pub mod ai_engine_integration;
pub mod cross_market_validation;
pub mod edge_lifecycle;
pub mod event_sourcing;
pub mod feature_discovery;
pub mod hypothesis_engine;
pub mod learning_engine_integration;
pub mod parameter_genetics;
pub mod research_lab;
pub mod research_memory;
pub mod strategy_generation;

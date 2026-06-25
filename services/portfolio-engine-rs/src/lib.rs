#![deny(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

pub mod allocation;
pub mod analytics;
pub mod api;
pub mod correlation;
pub mod drawdown;
pub mod exposure;
pub mod health;
pub mod heat;
pub mod portfolio;
pub mod quality;
pub mod recommendations;
pub mod storage;
pub mod rebalancing;
pub mod portfolio_optimizer;
pub mod diversification;
pub mod factor_exposure;
pub mod attribution;
pub mod capital_rotation;
pub mod account_allocator;
pub mod funded_account_manager;
pub mod prop_firm_constraints;
pub mod payout_management;
pub mod multi_account_orchestrator;
pub mod interceptors;
pub mod shadow;
pub mod validation;
pub mod integrations;
pub mod certification;

// APEX V3 Portfolio Engine
// Owns account-level intelligence, global exposure, and allocation.

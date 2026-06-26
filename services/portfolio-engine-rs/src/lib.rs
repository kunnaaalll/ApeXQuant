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
pub mod multi_account;
pub mod prop_firm;
pub mod optimization;
pub mod interceptors;
pub mod shadow;
pub mod validation;
pub mod integrations;
pub mod certification;
pub mod event_bus;
pub mod event_bus_subscriber;

// APEX V3 Portfolio Engine
// Owns account-level intelligence, global exposure, and allocation.

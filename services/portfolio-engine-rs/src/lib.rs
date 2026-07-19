#![allow(warnings, clippy::all, deprecated)]
#![deny(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

pub mod allocation;
pub mod analytics;
pub mod api;
pub mod certification;
pub mod config;
pub mod correlation;
pub mod drawdown;
pub mod event_bus;
pub mod event_bus_subscriber;
pub mod exposure;
pub mod health;
pub mod heat;
pub mod integrations;
pub mod interceptors;
pub mod multi_account;
pub mod optimization;
pub mod portfolio;
pub mod prop_firm;
pub mod quality;
pub mod rebalancing;
pub mod recommendations;
pub mod shadow;
pub mod storage;
pub mod validation;

// APEX V3 Portfolio Engine
// Owns account-level intelligence, global exposure, and allocation.

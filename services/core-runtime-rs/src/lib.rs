#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]

pub mod api;
pub mod configuration;
pub mod discovery;
pub mod event_sourcing;
pub mod governance;
pub mod health;
pub mod lifecycle;
pub mod observability;
pub mod orchestration;
pub mod service_registry;

// Phase 2 Modules
pub mod api_gateway;
pub mod cluster_state;
pub mod dependency_graph;
pub mod engine_registration;
pub mod governance_propagation;
pub mod heartbeat_engine;
pub mod observability_expansion;
pub mod recovery_engine;
pub mod service_mesh;

// Re-export common types
pub use configuration::*;
pub use governance::*;
pub use health::*;
pub use lifecycle::*;
pub use service_registry::*;

pub use api_gateway::*;
pub use cluster_state::*;
pub use dependency_graph::*;
pub use engine_registration::*;
pub use event_sourcing::*;
pub use governance_propagation::*;
pub use heartbeat_engine::*;
pub use observability_expansion::*;
pub use recovery_engine::*;
pub use service_mesh::*;

// Phase 3 Modules
pub mod autonomous_operations;
pub mod deployment_engine;
pub mod governance_runtime;
pub mod incident_engine;
pub mod observability_runtime;
pub mod recovery_orchestrator;
pub mod runtime_supervisor;
pub mod service_mesh_runtime;

pub use autonomous_operations::*;
pub use deployment_engine::*;
pub use governance_runtime::*;
pub use incident_engine::*;
pub use observability_runtime::*;
pub use recovery_orchestrator::*;
pub use runtime_supervisor::*;
pub use service_mesh_runtime::*;

// Wave 4 Modules
pub mod certification_report;
pub mod long_run_validation;
pub mod replay_certification;
pub mod restart_recovery;
pub mod shadow_metrics;
pub mod shadow_orchestrator;

pub use certification_report::*;
pub use long_run_validation::*;
pub use replay_certification::*;
pub use restart_recovery::*;
pub use shadow_metrics::*;
pub use shadow_orchestrator::*;

// Wave 5 Modules
pub mod broker_parity_monitoring;
pub mod broker_reconciliation;
pub mod broker_recovery;
pub mod duplicate_execution_prevention;
pub mod real_broker_certification;

pub use broker_parity_monitoring::*;
pub use broker_reconciliation::*;
pub use broker_recovery::*;
pub use duplicate_execution_prevention::*;
pub use real_broker_certification::*;

// Wave 7 Modules
pub mod shadow_campaign;
pub use shadow_campaign::*;

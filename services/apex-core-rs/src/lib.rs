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
pub mod engine_registration;
pub mod heartbeat_engine;
pub mod dependency_graph;
pub mod service_mesh;
pub mod recovery_engine;
pub mod cluster_state;
pub mod governance_propagation;
pub mod observability_expansion;
pub mod api_gateway;

// Re-export common types
pub use configuration::*;
pub use governance::*;
pub use health::*;
pub use lifecycle::*;
pub use service_registry::*;

pub use engine_registration::*;
pub use heartbeat_engine::*;
pub use dependency_graph::*;
pub use service_mesh::*;
pub use recovery_engine::*;
pub use cluster_state::*;
pub use governance_propagation::*;
pub use observability_expansion::*;
pub use event_sourcing::*;
pub use api_gateway::*;

// Phase 3 Modules
pub mod runtime_supervisor;
pub mod service_mesh_runtime;
pub mod deployment_engine;
pub mod recovery_orchestrator;
pub mod governance_runtime;
pub mod observability_runtime;
pub mod incident_engine;
pub mod autonomous_operations;

pub use runtime_supervisor::*;
pub use service_mesh_runtime::*;
pub use deployment_engine::*;
pub use recovery_orchestrator::*;
pub use governance_runtime::*;
pub use observability_runtime::*;
pub use incident_engine::*;
pub use autonomous_operations::*;

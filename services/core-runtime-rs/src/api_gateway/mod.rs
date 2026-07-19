use crate::governance_propagation::GovernancePolicy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTopologyResponse {
    pub active_nodes: u32,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphResponse {
    pub root_nodes: Vec<String>,
    pub dependencies: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthResponse {
    pub service_id: String,
    pub status: String,
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatusResponse {
    pub current_policy: GovernancePolicy,
    pub last_update_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartHistoryResponse {
    pub service_id: String,
    pub total_restarts: u32,
    pub last_restart_ms: u64,
}

use crate::cluster_state::ClusterStateManager;
use crate::dependency_graph::DependencyGraph;
use crate::governance_propagation::GovernancePropagator;
use crate::recovery_engine::RecoveryEngine;
use std::sync::Arc;

pub struct ApiGateway {
    cluster_state: Arc<ClusterStateManager>,
    dependency_graph: Arc<DependencyGraph>,
    recovery_engine: Arc<RecoveryEngine>,
    governance_propagator: Arc<GovernancePropagator>,
}

impl ApiGateway {
    pub fn new(
        cluster_state: Arc<ClusterStateManager>,
        dependency_graph: Arc<DependencyGraph>,
        recovery_engine: Arc<RecoveryEngine>,
        governance_propagator: Arc<GovernancePropagator>,
    ) -> Self {
        Self {
            cluster_state,
            dependency_graph,
            recovery_engine,
            governance_propagator,
        }
    }

    pub fn get_cluster_topology(&self) -> ClusterTopologyResponse {
        let metrics = self.cluster_state.get_cluster_metrics();
        ClusterTopologyResponse {
            active_nodes: metrics.active_services,
            services: vec![], // Not tracked by default HealthScore
        }
    }

    pub fn get_dependency_graph(&self) -> DependencyGraphResponse {
        let (roots, deps) = self.dependency_graph.get_topology_snapshot();
        DependencyGraphResponse {
            root_nodes: roots,
            dependencies: deps,
        }
    }

    pub fn get_service_health(&self, service_id: &str) -> ServiceHealthResponse {
        let state = self.cluster_state.get_service_state(service_id);
        ServiceHealthResponse {
            service_id: service_id.to_string(),
            status: state.to_string(),
            uptime_ms: 1000, // Real uptime would require tracking start time
        }
    }

    pub fn get_governance_status(&self) -> GovernanceStatusResponse {
        let policy = self.governance_propagator.get_current_policy();
        GovernanceStatusResponse {
            current_policy: policy,
            last_update_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_millis() as u64,
        }
    }

    pub fn get_restart_history(&self, service_id: &str) -> RestartHistoryResponse {
        let count = self.recovery_engine.get_restart_count(service_id);
        RestartHistoryResponse {
            service_id: service_id.to_string(),
            total_restarts: count,
            last_restart_ms: 0,
        }
    }

    pub fn get_prometheus_metrics(&self) -> String {
        let metrics = self.cluster_state.get_cluster_metrics();
        format!("# HELP apex_active_nodes Number of active nodes\n# TYPE apex_active_nodes gauge\napex_active_nodes {}\n", metrics.active_services)
    }
}

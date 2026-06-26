use crate::engine_registration::{EngineEndpoint, EngineRegistrar};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub target_service_id: String,
    pub endpoint: EngineEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub is_compatible: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub dependency_id: String,
    pub is_healthy: bool,
}

pub struct ServiceMesh<'a> {
    registrar: &'a EngineRegistrar,
}

impl<'a> ServiceMesh<'a> {
    pub fn new(registrar: &'a EngineRegistrar) -> Self {
        Self { registrar }
    }

    pub fn discover_service(&self, service_id: &str) -> Option<RoutingDecision> {
        let engine = self.registrar.get_engine(service_id)?;
        let endpoint = engine.endpoints.first()?.clone();
        Some(RoutingDecision {
            target_service_id: service_id.to_string(),
            endpoint,
        })
    }

    pub fn lookup_capability(&self, capability: &str) -> Vec<RoutingDecision> {
        let mut decisions = Vec::new();
        // this is O(n), ideally we'd index this, but for now it's fine
        for engine in self.registrar.capability_registry.capabilities_by_service.keys() {
            if let Some(caps) = self.registrar.capability_registry.get_capabilities(engine) {
                if caps.contains(&capability.to_string()) {
                    if let Some(decision) = self.discover_service(engine) {
                        decisions.push(decision);
                    }
                }
            }
        }
        decisions
    }

    pub fn check_version_compatibility(&self, service_a: &str, service_b: &str) -> CompatibilityReport {
        let mut issues = Vec::new();
        let engine_a = self.registrar.get_engine(service_a);
        let engine_b = self.registrar.get_engine(service_b);

        if let (Some(ea), Some(eb)) = (&engine_a, &engine_b) {
            // Extremely naive check: require exact version match.
            // In reality, this would use semantic versioning parsing.
            let is_compatible = ea.version == eb.version;
            if !is_compatible {
                issues.push("Version mismatch".to_string());
            }

            CompatibilityReport {
                is_compatible,
                issues,
            }
        } else {
            issues.push("One or both services not found".to_string());
            CompatibilityReport {
                is_compatible: false,
                issues,
            }
        }
    }
}

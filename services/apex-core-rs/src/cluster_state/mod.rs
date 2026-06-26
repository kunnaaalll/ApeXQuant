use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealthScore {
    pub score: u32, // 0-100
    pub active_services: u32,
    pub degraded_services: u32,
    pub unavailable_services: u32,
    pub maintenance_services: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRiskScore {
    pub risk_level: u32, // 0-100
    pub critical_dependencies_missing: u32,
    pub cascading_failure_probability: u32, // 0-100
}

#[derive(Default, Debug)]
pub struct ClusterStateManager {
    active_services: u32,
    degraded_services: u32,
    unavailable_services: u32,
    maintenance_services: u32,
}

impl ClusterStateManager {
    pub fn new() -> Self {
        Self {
            active_services: 0,
            degraded_services: 0,
            unavailable_services: 0,
            maintenance_services: 0,
        }
    }

    pub fn update_service_counts(
        &mut self,
        active: u32,
        degraded: u32,
        unavailable: u32,
        maintenance: u32,
    ) {
        self.active_services = active;
        self.degraded_services = degraded;
        self.unavailable_services = unavailable;
        self.maintenance_services = maintenance;
    }

    pub fn generate_health_score(&self) -> ClusterHealthScore {
        let total = self.active_services + self.degraded_services + self.unavailable_services + self.maintenance_services;
        
        let score = if total == 0 {
            0
        } else {
            let active_weight = self.active_services * 100;
            let degraded_weight = self.degraded_services * 50;
            (active_weight + degraded_weight).checked_div(total).unwrap_or(0)
        };

        ClusterHealthScore {
            score,
            active_services: self.active_services,
            degraded_services: self.degraded_services,
            unavailable_services: self.unavailable_services,
            maintenance_services: self.maintenance_services,
        }
    }

    pub fn generate_risk_score(&self, critical_missing: u32) -> ClusterRiskScore {
        let mut risk_level = 0;
        let mut cascading_probability = 0;

        if self.unavailable_services > 0 {
            risk_level += 30;
            cascading_probability += 20;
        }

        if critical_missing > 0 {
            risk_level += 70;
            cascading_probability += 80;
        }

        if self.degraded_services > 0 {
            risk_level += 10;
            cascading_probability += 10;
        }

        // Cap at 100
        risk_level = risk_level.min(100);
        cascading_probability = cascading_probability.min(100);

        ClusterRiskScore {
            risk_level,
            critical_dependencies_missing: critical_missing,
            cascading_failure_probability: cascading_probability,
        }
    }
}

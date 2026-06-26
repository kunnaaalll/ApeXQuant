use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    ServiceStarted(String),
    ServiceStopped(String),
    ServiceFailed(String),
    RecoveryTriggered(String),
    GovernanceChanged(String),
    // New Phase 2 Events
    EngineRegistered(String),
    EngineHeartbeatReceived(String),
    EngineRestarted(String),
    DependencyFailed(String),
    GovernancePropagated(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub sequence_number: u64,
    pub state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStateSnapshot {
    pub system_snapshot: SystemSnapshot,
    pub active_nodes: u32,
    pub degraded_nodes: u32,
    pub maintenance_nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySnapshot {
    pub system_snapshot: SystemSnapshot,
    pub healthy_dependencies: u32,
    pub failed_dependencies: u32,
}

#[derive(Default, Debug)]
pub struct EventStore {
    events: Vec<SystemEvent>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn append(&mut self, event: SystemEvent) -> Result<(), &'static str> {
        self.events.push(event);
        Ok(())
    }

    pub fn get_events(&self) -> &[SystemEvent] {
        &self.events
    }

    pub fn take_system_snapshot(&self) -> Result<SystemSnapshot, &'static str> {
        let serialized = serde_json::to_string(&self.events)
            .map_err(|_| "Failed to serialize events for hashing")?;
        
        let hash = ring::digest::digest(&ring::digest::SHA256, serialized.as_bytes());
        let state_hash = hash.as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        Ok(SystemSnapshot {
            sequence_number: self.events.len() as u64,
            state_hash,
        })
    }

    pub fn take_cluster_snapshot(&self, active: u32, degraded: u32, maintenance: u32) -> Result<ClusterStateSnapshot, &'static str> {
        Ok(ClusterStateSnapshot {
            system_snapshot: self.take_system_snapshot()?,
            active_nodes: active,
            degraded_nodes: degraded,
            maintenance_nodes: maintenance,
        })
    }

    pub fn take_dependency_snapshot(&self, healthy: u32, failed: u32) -> Result<DependencySnapshot, &'static str> {
        Ok(DependencySnapshot {
            system_snapshot: self.take_system_snapshot()?,
            healthy_dependencies: healthy,
            failed_dependencies: failed,
        })
    }
}

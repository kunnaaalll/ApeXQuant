use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessState {
    NotReady,
    Ready,
    ShuttingDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRegistration {
    pub service_id: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<EngineEndpoint>,
    pub supported_topics: Vec<String>,
    pub readiness_state: ReadinessState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHeartbeat {
    pub service_id: String,
    pub timestamp_ms: u64,
}

#[derive(Default, Debug)]
pub struct EngineCapabilityRegistry {
    pub capabilities_by_service: HashMap<String, Vec<String>>,
}

impl EngineCapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities_by_service: HashMap::new(),
        }
    }

    pub fn register(&mut self, service_id: &str, capabilities: Vec<String>) {
        self.capabilities_by_service
            .insert(service_id.to_string(), capabilities);
    }

    pub fn get_capabilities(&self, service_id: &str) -> Option<&Vec<String>> {
        self.capabilities_by_service.get(service_id)
    }
}

#[derive(Default, Debug)]
pub struct EngineLeaseManager {
    leases: HashMap<String, u64>,
}

impl EngineLeaseManager {
    pub fn new() -> Self {
        Self {
            leases: HashMap::new(),
        }
    }

    pub fn renew_lease(&mut self, service_id: &str, timestamp_ms: u64) {
        self.leases.insert(service_id.to_string(), timestamp_ms);
    }

    pub fn get_last_lease(&self, service_id: &str) -> Option<u64> {
        self.leases.get(service_id).copied()
    }
}

#[derive(Default, Debug)]
pub struct EngineRegistrar {
    engines: HashMap<String, EngineRegistration>,
    pub capability_registry: EngineCapabilityRegistry,
    pub lease_manager: EngineLeaseManager,
}

impl EngineRegistrar {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            capability_registry: EngineCapabilityRegistry::new(),
            lease_manager: EngineLeaseManager::new(),
        }
    }

    pub fn register(
        &mut self,
        reg: EngineRegistration,
        timestamp_ms: u64,
    ) -> Result<(), &'static str> {
        let id = reg.service_id.clone();
        if self.engines.contains_key(&id) {
            return Err("Engine already registered");
        }
        self.capability_registry
            .register(&id, reg.capabilities.clone());
        self.lease_manager.renew_lease(&id, timestamp_ms);
        self.engines.insert(id, reg);
        Ok(())
    }

    pub fn update_readiness(
        &mut self,
        service_id: &str,
        state: ReadinessState,
    ) -> Result<(), &'static str> {
        if let Some(engine) = self.engines.get_mut(service_id) {
            engine.readiness_state = state;
            Ok(())
        } else {
            Err("Engine not found")
        }
    }

    pub fn get_engine(&self, service_id: &str) -> Option<&EngineRegistration> {
        self.engines.get(service_id)
    }
}

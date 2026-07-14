use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Starting,
    Healthy,
    Warning,
    Degraded,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub identity: ServiceIdentity,
    pub endpoints: Vec<Endpoint>,
    pub capabilities: Vec<String>,
    pub state: ServiceState,
}

#[derive(Default, Debug)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceRegistration>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, registration: ServiceRegistration) -> Result<(), &'static str> {
        let id = registration.identity.id.clone();
        if self.services.contains_key(&id) {
            return Err("Service already registered");
        }
        let _ = self.services.insert(id, registration);
        Ok(())
    }

    pub fn update_state(&mut self, id: &str, state: ServiceState) -> Result<(), &'static str> {
        if let Some(svc) = self.services.get_mut(id) {
            svc.state = state;
            Ok(())
        } else {
            Err("Service not found")
        }
    }

    pub fn get_service(&self, id: &str) -> Option<&ServiceRegistration> {
        self.services.get(id)
    }

    pub fn total_services(&self) -> usize {
        self.services.len()
    }
}

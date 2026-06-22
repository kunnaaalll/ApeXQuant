use crate::brokers::broker::BrokerAdapter;
use crate::brokers::errors::BrokerError;
use crate::brokers::registry::selector::BrokerRole;
use crate::brokers::registry::failover::FailoverState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BrokerRegistry {
    pub adapters: HashMap<String, Arc<dyn BrokerAdapter>>,
    pub roles: HashMap<String, BrokerRole>,
    pub failover_state: RwLock<FailoverState>,
}

impl Default for BrokerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BrokerRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            roles: HashMap::new(),
            failover_state: RwLock::new(FailoverState::Healthy),
        }
    }

    pub fn register(&mut self, id: String, adapter: Arc<dyn BrokerAdapter>, role: BrokerRole) {
        self.adapters.insert(id.clone(), adapter);
        self.roles.insert(id, role);
    }

    pub async fn get_primary(&self) -> Result<Arc<dyn BrokerAdapter>, BrokerError> {
        let state = *self.failover_state.read().await;
        if state == FailoverState::Failover || state == FailoverState::Recovery {
            // In failover, we might want to route to Secondary
            for (id, role) in &self.roles {
                if *role == BrokerRole::Secondary {
                    if let Some(adapter) = self.adapters.get(id) {
                        return Ok(Arc::clone(adapter));
                    }
                }
            }
        }

        // Default: Primary
        for (id, role) in &self.roles {
            if *role == BrokerRole::Primary {
                if let Some(adapter) = self.adapters.get(id) {
                    return Ok(Arc::clone(adapter));
                }
            }
        }
        
        Err(BrokerError::InternalError("No suitable broker available".to_string()))
    }
}

pub struct ExecutionRouter {
    pub registry: Arc<BrokerRegistry>,
}

impl ExecutionRouter {
    pub fn new(registry: Arc<BrokerRegistry>) -> Self {
        Self { registry }
    }

    pub async fn route(&self) -> Result<Arc<dyn BrokerAdapter>, BrokerError> {
        self.registry.get_primary().await
    }
}

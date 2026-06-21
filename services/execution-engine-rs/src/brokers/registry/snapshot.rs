use crate::brokers::registry::failover::FailoverState;
use crate::brokers::registry::selector::BrokerRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistrySnapshot {
    pub failover_state: FailoverState,
    pub roles: HashMap<String, BrokerRole>,
    pub timestamp: i64,
}

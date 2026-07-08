use crate::brokers::connection::ConnectionState;
use crate::brokers::health::BrokerHealth;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mt5Health {
    pub connection_state: ConnectionState,
    pub core_health: BrokerHealth,
}

impl Mt5Health {
    pub fn new(connection_state: ConnectionState, core_health: BrokerHealth) -> Self {
        Self {
            connection_state,
            core_health,
        }
    }
}

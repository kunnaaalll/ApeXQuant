use crate::brokers::health::BrokerHealth;
use crate::brokers::connection::ConnectionState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceHealth {
    pub connection_state: ConnectionState,
    pub core_health: BrokerHealth,
    pub weight_usage: u32,
}

impl BinanceHealth {
    pub fn new(connection_state: ConnectionState, core_health: BrokerHealth, weight_usage: u32) -> Self {
        Self {
            connection_state,
            core_health,
            weight_usage,
        }
    }
}

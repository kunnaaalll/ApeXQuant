use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrokerHealth {
    pub latency_ms: Decimal,
    pub uptime_percentage: Decimal,
    pub heartbeat_interval_ms: Decimal,
    pub last_response_time: SystemTime,
    pub reconnect_attempts: u32,
}

impl BrokerHealth {
    pub fn new(latency_ms: Decimal, uptime_percentage: Decimal, heartbeat_interval_ms: Decimal, last_response_time: SystemTime, reconnect_attempts: u32) -> Self {
        Self {
            latency_ms,
            uptime_percentage,
            heartbeat_interval_ms,
            last_response_time,
            reconnect_attempts,
        }
    }

    pub fn is_healthy(&self) -> bool {
        use rust_decimal_macros::dec;
        self.uptime_percentage >= dec!(95.0) && self.latency_ms < dec!(500.0)
    }

    pub fn is_degraded(&self) -> bool {
        use rust_decimal_macros::dec;
        self.uptime_percentage < dec!(95.0) || self.latency_ms >= dec!(500.0)
    }
}

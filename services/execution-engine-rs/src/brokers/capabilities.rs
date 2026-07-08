use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BrokerCapabilities {
    pub supports_hedging: bool,
    pub supports_crypto: bool,
    pub supports_forex: bool,
    pub supports_fractional_lots: bool,
}

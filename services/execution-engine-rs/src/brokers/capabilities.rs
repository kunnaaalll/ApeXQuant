use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrokerCapabilities {
    pub supports_hedging: bool,
    pub supports_crypto: bool,
    pub supports_forex: bool,
    pub supports_fractional_lots: bool,
}

impl Default for BrokerCapabilities {
    fn default() -> Self {
        Self {
            supports_hedging: false,
            supports_crypto: false,
            supports_forex: false,
            supports_fractional_lots: false,
        }
    }
}

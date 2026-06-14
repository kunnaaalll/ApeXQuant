use crate::error::RiskError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Risk Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEngineConfig {
    /// Maximum risk per trade as percentage
    pub max_risk_percent: Decimal,
    /// Default risk per trade
    pub default_risk_percent: Decimal,
    /// Daily loss limit percentage
    pub daily_loss_limit_percent: Decimal,
    /// Maximum simultaneous trades
    pub max_simultaneous_trades: u32,
    /// Kelly fraction (if using Kelly sizing)
    pub kelly_fraction: Decimal,
    /// Enable shadow mode
    pub shadow_mode: bool,
    /// gRPC host
    pub grpc_host: String,
    /// gRPC port
    pub grpc_port: u16,
    /// Health HTTP port
    pub health_port: u16,
}

impl Default for RiskEngineConfig {
    fn default() -> Self {
        Self {
            max_risk_percent: Decimal::from_str_exact("0.02").unwrap_or(Decimal::new(2, 2)),
            default_risk_percent: Decimal::from_str_exact("0.01").unwrap_or(Decimal::new(1, 2)),
            daily_loss_limit_percent: Decimal::from_str_exact("0.05").unwrap_or(Decimal::new(5, 2)),
            max_simultaneous_trades: 5,
            kelly_fraction: Decimal::from_str_exact("0.25").unwrap_or(Decimal::new(25, 2)),
            shadow_mode: true,
            grpc_host: "0.0.0.0".to_string(),
            grpc_port: 50051,
            health_port: 8080,
        }
    }
}

impl RiskEngineConfig {
    /// Load configuration from environment and files
    pub fn load() -> Result<Self, RiskError> {
        Ok(Self::default())
    }
}

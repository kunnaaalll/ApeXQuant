//! Risk Engine gRPC Integration
//!
//! Real gRPC client that calls `RiskEngine::get_risk_state`.
//! No hardcoded "SAFE" fallback — errors propagate to callers.

use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use serde::{Deserialize, Serialize};

use apex_protos::risk::risk_engine_client::RiskEngineClient;
use apex_protos::risk::RiskStateQuery;
use tonic::transport::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub account_id: String,
    pub risk_score: Decimal,
    pub margin_utilization: Decimal,
    pub portfolio_heat: Decimal,
    pub status: String,
    pub can_trade: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskInterventionEvent {
    pub account_id: String,
    pub action: String,
    pub reason: String,
    pub timestamp: i64,
}

pub struct RiskClient {
    client: RiskEngineClient<Channel>,
}

impl RiskClient {
    /// Connect to the Risk Engine gRPC endpoint.
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = RiskEngineClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to risk engine: {}", e))?;
        Ok(Self { client })
    }

    /// Fetch real-time risk assessment for an account.
    ///
    /// Calls `RiskEngine::GetRiskState` and maps the proto response.
    /// Returns an error (never a fake "SAFE" value) if the call fails.
    pub async fn fetch_risk_assessment(&mut self, account_id: &str) -> Result<RiskAssessment, String> {
        let request = tonic::Request::new(RiskStateQuery {
            account_id: account_id.to_string(),
        });

        let response = self.client
            .get_risk_state(request)
            .await
            .map_err(|e| format!("RiskEngine::GetRiskState failed for {}: {}", account_id, e))?;

        let state = response.into_inner();

        // The new RiskStateResponse only returns `state` as a string (e.g. "ACTIVE", "BLOCKED")
        // and doesn't contain the detailed metrics (risk_score, etc.). We map what we can.
        let can_trade = state.state != "BLOCKED";
        let status = state.state.clone();
        
        let risk_score = Decimal::ZERO;
        let margin_utilization = Decimal::ZERO;
        let portfolio_heat = Decimal::ZERO;

        Ok(RiskAssessment {
            account_id: account_id.to_string(),
            risk_score,
            margin_utilization,
            portfolio_heat,
            status,
            can_trade,
        })
    }
}

use tonic::transport::Channel;
use apex_protos::analytics::analytics_engine_client::AnalyticsEngineClient;

pub struct PricingClient {
    pub client: Option<AnalyticsEngineClient<Channel>>,
}

impl PricingClient {
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = AnalyticsEngineClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to pricing/analytics service: {}", e))?;
        Ok(Self { client: Some(client) })
    }

    pub fn new() -> Self {
        Self { client: None }
    }
}

impl Default for PricingClient {
    fn default() -> Self {
        Self::new()
    }
}

use apex_protos::analytics::analytics_engine_client::AnalyticsEngineClient;
use tonic::transport::Channel;

pub struct MarketDataClient {
    pub client: Option<AnalyticsEngineClient<Channel>>,
}

impl MarketDataClient {
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = AnalyticsEngineClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to market data service: {}", e))?;
        Ok(Self {
            client: Some(client),
        })
    }

    pub fn new() -> Self {
        Self { client: None }
    }
}

impl Default for MarketDataClient {
    fn default() -> Self {
        Self::new()
    }
}

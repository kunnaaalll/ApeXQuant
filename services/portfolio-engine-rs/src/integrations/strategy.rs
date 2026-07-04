use tonic::transport::Channel;
use apex_protos::strategy::strategy_service_client::StrategyServiceClient;

pub struct StrategyClient {
    pub client: Option<StrategyServiceClient<Channel>>,
}

impl StrategyClient {
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = StrategyServiceClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to strategy service: {}", e))?;
        Ok(Self { client: Some(client) })
    }

    pub fn new() -> Self {
        Self { client: None }
    }
}

impl Default for StrategyClient {
    fn default() -> Self {
        Self::new()
    }
}

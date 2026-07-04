use tonic::transport::Channel;
use apex_protos::learning::learning_engine_client::LearningEngineClient;

pub struct AiClient {
    pub client: Option<LearningEngineClient<Channel>>,
}

impl AiClient {
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = LearningEngineClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to AI/Learning engine: {}", e))?;
        Ok(Self { client: Some(client) })
    }

    pub fn new() -> Self {
        Self { client: None }
    }
}

impl Default for AiClient {
    fn default() -> Self {
        Self::new()
    }
}

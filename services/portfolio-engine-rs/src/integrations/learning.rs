use tonic::transport::Channel;
use apex_protos::learning::learning_engine_client::LearningEngineClient;

pub struct LearningClient {
    pub client: Option<LearningEngineClient<Channel>>,
}

impl LearningClient {
    pub async fn connect(url: String) -> Result<Self, String> {
        let client = LearningEngineClient::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to learning engine: {}", e))?;
        Ok(Self { client: Some(client) })
    }

    pub fn new() -> Self {
        Self { client: None }
    }
}

impl Default for LearningClient {
    fn default() -> Self {
        Self::new()
    }
}

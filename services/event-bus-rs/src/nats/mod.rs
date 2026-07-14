use async_nats::jetstream::{self, Context, stream::Stream};
use async_nats::Client;
use anyhow::{Result, Context as AnyhowContext};

#[derive(Clone)]
pub struct NatsManager {
    pub client: Client,
    pub jetstream: Context,
}

impl NatsManager {
    pub async fn connect(url: &str) -> Result<Self> {
        tracing::info!("Connecting to NATS at {}", url);
        let client = async_nats::connect(url)
            .await
            .context("Failed to connect to NATS")?;
        
        let jetstream = jetstream::new(client.clone());
        
        Ok(Self {
            client,
            jetstream,
        })
    }

    pub async fn get_or_create_stream(&self, stream_name: &str, subjects: Vec<String>) -> Result<Stream> {
        use async_nats::jetstream::stream::Config;
        
        tracing::info!("Ensuring stream {} exists for subjects {:?}", stream_name, subjects);
        
        let stream = self.jetstream.get_or_create_stream(Config {
            name: stream_name.to_string(),
            subjects,
            max_bytes: 1024 * 1024 * 1024, // 1 GB max size
            max_age: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            ..Default::default()
        }).await.context("Failed to get or create JetStream stream")?;
        
        Ok(stream)
    }
}

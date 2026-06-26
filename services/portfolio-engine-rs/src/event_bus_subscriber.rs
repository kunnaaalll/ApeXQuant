use apex_protos::events::{Event, SubscribeRequest, event_bus_service_client::EventBusServiceClient, EventBatch, AckRequest, AckResponse};
use anyhow::{Result, Context};
use tokio::sync::mpsc;
use tonic::transport::Channel;
use tracing::{info, warn};

#[derive(Clone)]
pub struct EventBusSubscriber {
    client: EventBusServiceClient<Channel>,
    consumer_group: String,
    consumer_id: String,
}

impl EventBusSubscriber {
    pub async fn connect(url: String, consumer_group: String, consumer_id: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(url).await
            .context("Failed to connect to EventBusService")?;
        Ok(Self { client, consumer_group, consumer_id })
    }

    pub async fn subscribe(
        &self, 
        topic: &str, 
    ) -> Result<mpsc::Receiver<Event>> {
        let (tx, rx) = mpsc::channel(100);
        
        let req = SubscribeRequest {
            topics: vec![topic.to_string()],
            consumer_id: self.consumer_id.clone(),
            consumer_group: self.consumer_group.clone(),
            start_from: None,
            max_batch_size: 100,
            max_wait_ms: Some(apex_protos::common::Duration {
                seconds: 0,
                nanos: 50_000_000,
            }),
            filter: None,
        };

        let mut client = self.client.clone();
        let request = tonic::Request::new(req);
        let mut stream = client.subscribe(request).await?.into_inner();

        tokio::spawn(async move {
            while let Ok(Some(batch)) = stream.message().await {
                for event in batch.events {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    pub async fn ack(&self, event_ids: Vec<String>) -> Result<AckResponse> {
        let mut client = self.client.clone();
        let req = AckRequest {
            consumer_group: self.consumer_group.clone(),
            consumer_id: self.consumer_id.clone(),
            event_ids,
            failed: vec![],
        };
        
        let response = client.ack(tonic::Request::new(req)).await
            .context("Failed to ack events")?;
            
        Ok(response.into_inner())
    }

    pub async fn start_listening(&self, mut rx: mpsc::Receiver<Event>) {
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Determine event type from string property or payload
                let event_type = event.event_type.as_str();
                match event_type {
                    "StrategyPerformanceEvent" => {
                        info!("Received StrategyPerformanceEvent");
                        // Process strategy performance
                    }
                    "ExecutionQualityEvent" => {
                        info!("Received ExecutionQualityEvent");
                        // Process execution quality
                    }
                    "RiskInterventionEvent" => {
                        info!("Received RiskInterventionEvent");
                        // Process risk intervention
                    }
                    "MarketRegimeEvent" => {
                        info!("Received MarketRegimeEvent");
                        // Process market regime change
                    }
                    "LearningRecommendationEvent" => {
                        info!("Received LearningRecommendationEvent");
                        // Process learning recommendation
                    }
                    "AiAllocationRecommendationEvent" => {
                        info!("Received AiAllocationRecommendationEvent");
                        // Process AI recommendation
                    }
                    _ => {
                        warn!("Received unknown event type: {}", event_type);
                    }
                }
            }
        });
    }
}

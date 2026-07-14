pub use apex_protos::events::*;
use apex_protos::events::{
    EventBatch, SubscribeRequest, AckRequest, AckResponse
};
use apex_protos::events::event_bus_service_client::EventBusServiceClient;
use tonic::transport::Channel;
use anyhow::{Result, Context};

pub struct EventBusSubscriber {
    client: EventBusServiceClient<Channel>,
    consumer_group: String,
    consumer_id: String,
}

impl EventBusSubscriber {
    pub async fn connect(endpoint: String, consumer_group: String, consumer_id: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(endpoint).await
            .context("Failed to connect subscriber to GRPC endpoint")?;
        
        Ok(Self {
            client,
            consumer_group,
            consumer_id,
        })
    }

    pub async fn subscribe(
        &mut self, 
        topics: Vec<String>, 
        start_from: Option<StreamPosition>,
        filter: Option<FilterExpression>
    ) -> Result<tonic::Streaming<EventBatch>> {
        let request = SubscribeRequest {
            consumer_group: self.consumer_group.clone(),
            consumer_id: self.consumer_id.clone(),
            topics,
            start_from,
            max_batch_size: 100,
            max_wait_ms: None,
            filter,
        };

        let response = self.client.subscribe(request).await
            .context("Failed to subscribe via GRPC")?;
            
        Ok(response.into_inner())
    }

    pub async fn ack(&mut self, event_ids: Vec<String>) -> Result<AckResponse> {
        let request = AckRequest {
            consumer_group: self.consumer_group.clone(),
            consumer_id: self.consumer_id.clone(),
            event_ids,
            failed: vec![],
        };

        let response = self.client.ack(request).await
            .context("Failed to send ack via GRPC")?;
            
        Ok(response.into_inner())
    }
}

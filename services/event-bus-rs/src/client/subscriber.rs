use apex_protos::events::{
    Event, SubscribeRequest, StreamPosition, FilterExpression, EventBatch, AckRequest, AckResponse
};
use apex_protos::events::event_bus_service_client::EventBusServiceClient;
use tonic::transport::Channel;
use anyhow::{Result, Context};
use std::time::Instant;

pub struct EventBusSubscriber {
    client: EventBusServiceClient<Channel>,
    consumer_group: String,
    consumer_id: String,
}

impl EventBusSubscriber {
    pub async fn connect(endpoint: String, consumer_group: String, consumer_id: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(endpoint).await?;
        
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

        let response = self.client.subscribe(request).await?;
        Ok(response.into_inner())
    }

    pub async fn ack(&mut self, event_ids: Vec<String>) -> Result<AckResponse> {
        let request = AckRequest {
            consumer_group: self.consumer_group.clone(),
            consumer_id: self.consumer_id.clone(),
            event_ids,
            failed: vec![],
        };

        let response = self.client.ack(request).await?;
        Ok(response.into_inner())
    }
}

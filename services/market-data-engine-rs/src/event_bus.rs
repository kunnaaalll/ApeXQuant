use anyhow::{Context, Result};
use apex_protos::events::{
    event_bus_service_client::EventBusServiceClient, Event, PublishRequest, PublishResponse,
};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct EventBusPublisher {
    client: EventBusServiceClient<Channel>,
}

impl EventBusPublisher {
    pub async fn connect(url: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(url)
            .await
            .context("Failed to connect to EventBusService")?;
        Ok(Self { client })
    }

    pub async fn publish(&self, event: Event) -> Result<PublishResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(PublishRequest {
            events: vec![event],
            r#async: false,
            durability: apex_protos::events::DurabilityLevel::DurabilityDisk as i32,
        });

        let response = client
            .publish(request)
            .await
            .context("Failed to publish event")?;

        Ok(response.into_inner())
    }

    pub async fn publish_batch(&self, events: Vec<Event>) -> Result<PublishResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(PublishRequest {
            events,
            r#async: false,
            durability: apex_protos::events::DurabilityLevel::DurabilityDisk as i32,
        });

        let response = client
            .publish(request)
            .await
            .context("Failed to publish batch")?;

        Ok(response.into_inner())
    }
}

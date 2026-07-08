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

#[derive(Clone)]
pub struct EventBusSubscriber {
    client: EventBusServiceClient<Channel>,
}

impl EventBusSubscriber {
    pub async fn connect(url: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(url)
            .await
            .context("Failed to connect to EventBusService for subscription")?;
        Ok(Self { client })
    }

    pub async fn subscribe(
        &self,
        consumer_group: String,
        consumer_id: String,
        topics: Vec<String>,
    ) -> Result<tonic::codec::Streaming<apex_protos::events::EventBatch>> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(apex_protos::events::SubscribeRequest {
            consumer_group,
            consumer_id,
            topics,
            start_from: None,
            max_batch_size: 100,
            max_wait_ms: None,
            filter: None,
        });

        let response = client
            .subscribe(request)
            .await
            .context("Failed to subscribe to events")?;

        Ok(response.into_inner())
    }
}

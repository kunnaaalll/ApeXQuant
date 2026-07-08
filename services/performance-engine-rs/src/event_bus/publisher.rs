use anyhow::{Context, Result};
use apex_protos::events::event_bus_service_client::EventBusServiceClient;
use apex_protos::events::{Event, PublishRequest};
use tonic::transport::Channel;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct EventBusPublisher {
    client: EventBusServiceClient<Channel>,
    service_name: String,
}

impl EventBusPublisher {
    pub async fn connect(url: String, service_name: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(url).await
            .context("Failed to connect to EventBusService for publishing")?;
        Ok(Self { client, service_name })
    }

    /// Build a standard event envelope around a payload
    pub fn build_event<T>(&self, topic: &str, causation_id: Option<String>, payload: apex_protos::events::event::Payload) -> Event {
        let now = Utc::now();
        Event {
            event_id: Some(apex_protos::common::Uuid {
                value: Uuid::new_v4().into_bytes().to_vec(),
            }),
            spec_version: Some(apex_protos::common::SemanticVersion {
                major: 1, minor: 0, patch: 0, pre_release: String::new(), build: String::new(),
            }),
            occurred_at: Some(apex_protos::common::Timestamp {
                seconds: now.timestamp(),
                nanos: now.timestamp_subsec_nanos() as i32,
            }),
            published_at: None, // Set by event bus
            event_type: "performance_event".to_string(), // In reality we'd pull this from the Payload enum variant
            source_service: self.service_name.clone(),
            topic: topic.to_string(),
            correlation: None,
            causation_id: causation_id.unwrap_or_default(),
            deduplication_key: Uuid::new_v4().to_string(),
            payload_hash: vec![],
            payload: Some(payload),
        }
    }

    /// Publish one or more events to the bus
    pub async fn publish(&self, events: Vec<Event>) -> Result<()> {
        let mut client = self.client.clone();
        let req = PublishRequest { 
            events,
            r#async: false,
            durability: 0,
        };
        client.publish(tonic::Request::new(req)).await
            .context("Failed to publish events")?;
        Ok(())
    }
}

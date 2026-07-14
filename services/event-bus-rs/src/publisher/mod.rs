use apex_protos::events::{Event, PublishResponse};
use anyhow::{Result, Context};
use crate::nats::NatsManager;
use crate::router::Router;

#[derive(Clone)]
pub struct EventBusPublisher {
    nats: NatsManager,
    router: Router,
}

impl EventBusPublisher {
    pub fn new(nats: NatsManager, router: Router) -> Self {
        Self { nats, router }
    }

    pub async fn publish(&self, event: Event) -> Result<PublishResponse> {
        let topic = self.router.route_event(&event).await?;
        let payload = prost::Message::encode_to_vec(&event);
        
        self.nats.client.publish(topic, payload.into()).await
            .context("Failed to publish event to NATS")?;
            
        let event_id = event.event_id.as_ref()
            .map(|id| String::from_utf8_lossy(&id.value).into_owned())
            .ok_or_else(|| anyhow::anyhow!("Missing event ID"))?;

        Ok(PublishResponse {
            result: Some(apex_protos::common::Result {
                ok: true,
                error: None,
            }),
            published_ids: vec![event_id],
        })
    }

    pub async fn publish_batch(&self, events: Vec<Event>) -> Result<Vec<PublishResponse>> {
        let mut acks = Vec::with_capacity(events.len());
        for event in events {
            acks.push(self.publish(event).await?);
        }
        Ok(acks)
    }
}

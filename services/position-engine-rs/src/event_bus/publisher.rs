use async_nats::Client;
use bytes::Bytes;
use tracing::info;
use super::{PositionEventPayload, PositionHealthEventPayload, PositionAnalyticsEventPayload};

#[derive(Clone)]
pub struct EventPublisher {
    client: Client,
}

impl EventPublisher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn publish_position_created(&self, payload: &PositionEventPayload) -> anyhow::Result<()> {
        let data: Bytes = serde_json::to_vec(payload)?.into();
        self.client.publish("position.created", data).await?;
        info!("Published position.created for position_id={}", payload.position_id);
        Ok(())
    }

    pub async fn publish_position_updated(&self, payload: &PositionEventPayload) -> anyhow::Result<()> {
        let data: Bytes = serde_json::to_vec(payload)?.into();
        self.client.publish("position.updated", data).await?;
        info!("Published position.updated for position_id={}", payload.position_id);
        Ok(())
    }

    pub async fn publish_position_closed(&self, payload: &PositionEventPayload) -> anyhow::Result<()> {
        let data: Bytes = serde_json::to_vec(payload)?.into();
        self.client.publish("position.closed", data).await?;
        info!("Published position.closed for position_id={}", payload.position_id);
        Ok(())
    }

    pub async fn publish_position_health_updated(&self, payload: &PositionHealthEventPayload) -> anyhow::Result<()> {
        let data: Bytes = serde_json::to_vec(payload)?.into();
        self.client.publish("position.health.updated", data).await?;
        Ok(())
    }

    pub async fn publish_position_analytics_updated(&self, payload: &PositionAnalyticsEventPayload) -> anyhow::Result<()> {
        let data: Bytes = serde_json::to_vec(payload)?.into();
        self.client.publish("position.analytics.updated", data).await?;
        Ok(())
    }
}


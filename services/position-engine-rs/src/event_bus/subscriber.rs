use anyhow::{Context, Result};
use apex_protos::events::{event::Payload, event_bus_service_client::EventBusServiceClient, SubscribeRequest};
use rust_decimal::Decimal;
use std::sync::Arc;
use tonic::transport::Channel;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::pnl::UnrealizedPnL;
use crate::positions::{PositionRegistry, PositionState, PositionTracker};

#[derive(Clone)]
pub struct EventSubscriber {
    client: EventBusServiceClient<Channel>,
    registry: Arc<PositionRegistry>,
}

impl EventSubscriber {
    pub async fn connect(url: String, registry: Arc<PositionRegistry>) -> Result<Self> {
        let client = EventBusServiceClient::connect(url)
            .await
            .context("failed to connect to apex event bus")?;
        Ok(Self { client, registry })
    }

    pub async fn start(&self) -> Result<()> {
        let request = SubscribeRequest {
            consumer_group: "position-engine".to_string(),
            consumer_id: format!("position-engine-{}", Uuid::new_v4()),
            topics: vec!["execution.order".to_string(), "position.*".to_string(), "market.tick".to_string()],
            start_from: None,
            max_batch_size: 100,
            max_wait_ms: Some(apex_protos::common::Duration { seconds: 0, nanos: 50_000_000 }),
            filter: None,
        };
        let mut stream = self.client.clone().subscribe(tonic::Request::new(request)).await?.into_inner();
        let registry = self.registry.clone();
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(batch)) => {
                        for event in batch.events {
                            if let Err(e) = apply_event(&registry, event) {
                                warn!(%e, "position event rejected");
                            }
                        }
                    }
                    Ok(None) => { error!("event bus subscription ended"); break; }
                    Err(e) => { error!(%e, "event bus subscription failed"); break; }
                }
            }
        });
        info!("Position Engine subscribed through apex-event-bus");
        Ok(())
    }
}

fn apply_event(registry: &PositionRegistry, event: apex_protos::events::Event) -> Result<()> {
    match event.payload {
        Some(Payload::OrderFilled(fill)) => {
            let position_id = Uuid::parse_str(&fill.position_id).context("invalid fill position id")?;
            let price = fill.fill_price.as_ref().and_then(|p| Decimal::from_str_exact(&p.value).ok()).context("invalid fill price")?;
            let size = fill.fill_volume.as_ref().and_then(|v| Decimal::from_str_exact(&v.units).ok()).context("invalid fill volume")?;
            if let Some(mut tracker) = registry.get(&position_id) {
                let total_cost = tracker.average_entry_price * tracker.current_size + price * size;
                tracker.current_size += size;
                if tracker.current_size > Decimal::ZERO { tracker.average_entry_price = total_cost / tracker.current_size; }
                tracker.state = PositionState::Active;
                registry.insert(tracker);
            }
        }
        Some(Payload::PositionOpened(opened)) => {
            let position_id = Uuid::parse_str(&opened.position_id).context("invalid position id")?;
            let price = opened.entry_price.as_ref().and_then(|p| Decimal::from_str_exact(&p.value).ok()).context("invalid entry price")?;
            let size = opened.initial_volume.as_ref().and_then(|v| Decimal::from_str_exact(&v.units).ok()).context("invalid position volume")?;
            let side = if opened.side == apex_protos::common::TradeSide::Buy as i32 { "buy" } else { "sell" };
            registry.insert(PositionTracker::new(position_id, opened.symbol, side.to_string(), size, price));
        }
        Some(Payload::PositionClosed(closed)) => {
            let position_id = Uuid::parse_str(&closed.position_id).context("invalid close position id")?;
            registry.remove(&position_id);
        }
        Some(Payload::TickReceived(tick)) => {
            let bid = tick.bid.as_ref().and_then(|p| Decimal::from_str_exact(&p.value).ok()).context("invalid bid")?;
            let ask = tick.ask.as_ref().and_then(|p| Decimal::from_str_exact(&p.value).ok()).context("invalid ask")?;
            let mid = (bid + ask) / Decimal::TWO;
            for mut entry in registry.positions.iter_mut() {
                let tracker = entry.value_mut();
                if tracker.symbol == tick.symbol && !matches!(tracker.state, PositionState::Closed | PositionState::Archived | PositionState::Invalid) {
                    tracker.update_price(mid);
                    tracker.unrealized_pnl = UnrealizedPnL::calculate(&tracker.side, mid, tracker.average_entry_price, tracker.current_size);
                }
            }
        }
        Some(Payload::OrderCancelled(_)) | Some(Payload::OrderRejected(_)) | None => {}
        _ => {}
    }
    Ok(())
}

use async_nats::Client;
use futures::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::pnl::UnrealizedPnL;
use crate::positions::{PositionRegistry, PositionState};

#[derive(Debug, Deserialize)]
struct MarketTickPayload {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
}

#[derive(Debug, Deserialize)]
struct OrderFilledPayload {
    pub position_id: String,
    pub fill_price: Decimal,
    pub fill_size: Decimal,
    pub side: String,
}

#[derive(Clone)]
pub struct EventSubscriber {
    client: Client,
    registry: Arc<PositionRegistry>,
}

impl EventSubscriber {
    pub fn new(client: Client, registry: Arc<PositionRegistry>) -> Self {
        Self { client, registry }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // --- Subscription 1: Execution Order events ---
        {
            let client = self.client.clone();
            let registry = self.registry.clone();
            tokio::spawn(async move {
                let mut sub = match client.subscribe("execution.order.*").await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to subscribe to execution.order.*: {}", e);
                        return;
                    }
                };
                info!("Subscribed to execution.order.*");

                while let Some(msg) = sub.next().await {
                    let subject = msg.subject.as_str();
                    debug!("Received event on {}", subject);

                    match subject {
                        s if s.ends_with(".filled") => {
                            match serde_json::from_slice::<OrderFilledPayload>(&msg.payload) {
                                Ok(payload) => {
                                    let Ok(pos_id) = Uuid::parse_str(&payload.position_id) else {
                                        warn!(
                                            "Invalid position_id in fill event: {}",
                                            payload.position_id
                                        );
                                        continue;
                                    };
                                    if let Some(mut tracker) = registry.get(&pos_id) {
                                        // Update average entry price on scale-in
                                        if tracker.state == PositionState::Opening
                                            || tracker.state == PositionState::ScalingIn
                                        {
                                            let total_cost = tracker.average_entry_price
                                                * tracker.current_size
                                                + payload.fill_price * payload.fill_size;
                                            tracker.current_size += payload.fill_size;
                                            if tracker.current_size > Decimal::ZERO {
                                                tracker.average_entry_price =
                                                    total_cost / tracker.current_size;
                                            }
                                            tracker.state = PositionState::Active;
                                        }
                                        registry.insert(tracker);
                                    } else {
                                        warn!("Received fill for unknown position {}", pos_id);
                                    }
                                }
                                Err(e) => error!("Failed to parse OrderFilledPayload: {}", e),
                            }
                        }
                        s if s.ends_with(".cancelled") || s.ends_with(".rejected") => {
                            info!("Order {} — no position state change needed", subject);
                        }
                        _ => {
                            debug!("Unhandled execution subject: {}", subject);
                        }
                    }
                }
            });
        }

        // --- Subscription 2: Market Tick events ---
        {
            let client = self.client.clone();
            let registry = self.registry.clone();
            tokio::spawn(async move {
                let mut sub = match client.subscribe("market.tick.*").await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to subscribe to market.tick.*: {}", e);
                        return;
                    }
                };
                info!("Subscribed to market.tick.*");

                while let Some(msg) = sub.next().await {
                    match serde_json::from_slice::<MarketTickPayload>(&msg.payload) {
                        Ok(tick) => {
                            // Mid price from bid/ask
                            let mid = (tick.bid + tick.ask) / Decimal::TWO;

                            // Update all open positions for this symbol
                            for mut entry in registry.positions.iter_mut() {
                                let tracker = entry.value_mut();
                                if tracker.symbol != tick.symbol {
                                    continue;
                                }
                                if matches!(
                                    tracker.state,
                                    PositionState::Closed
                                        | PositionState::Archived
                                        | PositionState::Invalid
                                ) {
                                    continue;
                                }
                                tracker.update_price(mid);
                                tracker.unrealized_pnl = UnrealizedPnL::calculate(
                                    &tracker.side,
                                    mid,
                                    tracker.average_entry_price,
                                    tracker.current_size,
                                );
                            }
                        }
                        Err(e) => error!("Failed to parse MarketTickPayload: {}", e),
                    }
                }
            });
        }

        Ok(())
    }
}

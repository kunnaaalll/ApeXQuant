use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::positions::{PositionRegistry, PositionState, PositionTracker};
use crate::storage::PostgresStore;

#[derive(serde::Deserialize, Debug)]
struct Mt5Position {
    ticket: u64,
    symbol: String,
    volume: f64,
    entry_price: f64,
    floating_pnl: f64,
    side: String,
    sl: f64,
    tp: f64,
}

#[derive(serde::Serialize)]
struct StopsModifyRequest {
    stop_loss: f64,
    take_profit: Option<f64>,
}

pub struct PositionManager {
    registry: PositionRegistry,
    store: Arc<PostgresStore>,
    mt5_bridge_url: String,
    client: reqwest::Client,
}

impl PositionManager {
    pub fn new(
        registry: PositionRegistry,
        store: Arc<PostgresStore>,
        mt5_bridge_url: String,
    ) -> Self {
        Self {
            registry,
            store,
            mt5_bridge_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1500));
            loop {
                interval.tick().await;
                if let Err(e) = self.sync_positions().await {
                    error!("Position Manager sync cycle failed: {:?}", e);
                }
            }
        });
    }

    async fn sync_positions(&self) -> anyhow::Result<()> {
        // 1. Fetch active positions from the MT5 python bridge
        let url = format!("{}/positions", self.mt5_bridge_url);
        let res = self.client.get(&url).send().await?;
        if !res.status().is_success() {
            warn!(
                "Failed to fetch positions from bridge: status={}",
                res.status()
            );
            return Ok(());
        }

        let active_mt5_positions: Vec<Mt5Position> = res.json().await?;
        let mut active_ids = std::collections::HashSet::new();

        // 2. Synchronize active positions
        for pos in &active_mt5_positions {
            // Generate deterministic UUID from MT5 ticket
            let mut bytes = [0u8; 16];
            bytes[..8].copy_from_slice(&pos.ticket.to_be_bytes());
            let position_id = Uuid::from_bytes(bytes);
            active_ids.insert(position_id);

            // Fetch from registry or load/create
            let mut tracker = if let Some(t) = self.registry.get(&position_id) {
                t
            } else {
                // Check if it exists in Postgres
                if let Ok(Some(db_pos)) = self.store.get_position(position_id).await {
                    info!(
                        "Loaded active position {} (ticket {}) from DB",
                        position_id, pos.ticket
                    );
                    self.registry.insert(db_pos.clone());
                    db_pos
                } else {
                    // Create new position tracker
                    let side_str = if pos.side.to_lowercase().contains("buy") {
                        "buy"
                    } else {
                        "sell"
                    };
                    let mut new_tracker = PositionTracker::new(
                        position_id,
                        pos.symbol.clone(),
                        side_str.to_string(),
                        Decimal::from_f64(pos.volume).unwrap_or(Decimal::ZERO),
                        Decimal::from_f64(pos.entry_price).unwrap_or(Decimal::ZERO),
                    );
                    new_tracker.state = PositionState::Active;
                    new_tracker.current_stop_loss = if pos.sl > 0.0 {
                        Decimal::from_f64(pos.sl)
                    } else {
                        None
                    };
                    new_tracker.initial_take_profit = if pos.tp > 0.0 {
                        Decimal::from_f64(pos.tp)
                    } else {
                        None
                    };

                    info!(
                        "Registered new position {} for symbol {} (ticket {})",
                        position_id, pos.symbol, pos.ticket
                    );
                    self.registry.insert(new_tracker.clone());
                    if let Err(e) = self.store.save_position(&new_tracker).await {
                        error!("Failed to save new position to DB: {:?}", e);
                    }
                    new_tracker
                }
            };

            // Update live metrics
            // We approximate current price based on entry + pnl, or calculate it.
            // For simplicity, we calculate tick price or read it from active metrics.
            let float_pnl = Decimal::from_f64(pos.floating_pnl).unwrap_or(Decimal::ZERO);
            tracker.unrealized_pnl = float_pnl;
            tracker.last_updated_at = SystemTime::now();

            // Set current prices
            let entry = tracker.initial_entry_price;
            let size = tracker.current_size;

            // EURUSD standard forex lot size is 100,000. JPY is 1,000. BTC is 1.
            let mult = if pos.symbol.contains("JPY") {
                dec!(1000.0)
            } else if pos.symbol.contains("BTC") {
                dec!(1.0)
            } else {
                dec!(100000.0)
            };
            let pnl_pips = if size.is_zero() {
                Decimal::ZERO
            } else {
                float_pnl / (size * mult)
            };

            let current_price = if pos.side.to_lowercase() == "buy" {
                entry + pnl_pips
            } else {
                entry - pnl_pips
            };
            tracker.current_price = current_price.normalize();

            // Handle Trailing Stop logic
            // Enable default trailing stop (e.g. 20 pips/points) for test positions
            let trailing_pips = dec!(0.0020); // 20 pips for standard Forex pairs
            let step_pips = dec!(0.0005); // 5 pips minimum movement before modifying stops

            if let Some(sl_val) = tracker.current_stop_loss {
                let should_modify = if tracker.side == "buy" {
                    let calculated_sl = current_price - trailing_pips;
                    if calculated_sl > sl_val && (calculated_sl - sl_val) >= step_pips {
                        Some(calculated_sl)
                    } else {
                        None
                    }
                } else {
                    let calculated_sl = current_price + trailing_pips;
                    if calculated_sl < sl_val && (sl_val - calculated_sl) >= step_pips {
                        Some(calculated_sl)
                    } else {
                        None
                    }
                };

                if let Some(new_sl) = should_modify {
                    info!(
                        "Trailing SL triggered for ticket {}: old_sl={}, new_sl={}",
                        pos.ticket, sl_val, new_sl
                    );

                    // Call the stop modify endpoint
                    let stops_url =
                        format!("{}/positions/{}/stops", self.mt5_bridge_url, pos.ticket);
                    let stops_req = StopsModifyRequest {
                        stop_loss: new_sl.to_f64().unwrap_or(0.0),
                        take_profit: tracker.initial_take_profit.and_then(|tp| tp.to_f64()),
                    };

                    match self.client.post(&stops_url).json(&stops_req).send().await {
                        Ok(stops_res) => {
                            if stops_res.status().is_success() {
                                info!(
                                    "Stops successfully modified on MT5 for ticket {}",
                                    pos.ticket
                                );
                                tracker.current_stop_loss = Some(new_sl);
                                tracker.last_updated_at = SystemTime::now();
                            } else {
                                warn!(
                                    "Broker stops modification rejected: status={}",
                                    stops_res.status()
                                );
                            }
                        }
                        Err(e) => {
                            error!("HTTP request to stops endpoint failed: {:?}", e);
                        }
                    }
                }
            } else {
                // If there's no stop loss yet, let's initialize a trailing SL at entry - 20 pips to protect capital!
                let initial_sl = if tracker.side == "buy" {
                    current_price - trailing_pips
                } else {
                    current_price + trailing_pips
                };

                info!(
                    "Initializing trailing SL for ticket {} at {}",
                    pos.ticket, initial_sl
                );
                let stops_url = format!("{}/positions/{}/stops", self.mt5_bridge_url, pos.ticket);
                let stops_req = StopsModifyRequest {
                    stop_loss: initial_sl.to_f64().unwrap_or(0.0),
                    take_profit: tracker.initial_take_profit.and_then(|tp| tp.to_f64()),
                };

                if let Ok(stops_res) = self.client.post(&stops_url).json(&stops_req).send().await {
                    if stops_res.status().is_success() {
                        tracker.current_stop_loss = Some(initial_sl);
                    }
                }
            }

            // Sync changes back to Registry & DB
            self.registry.insert(tracker.clone());
            let _ = self.store.save_position(&tracker).await;
        }

        // 3. Detect closed positions
        // Gather all registry positions
        let mut closed_ids = Vec::new();
        // Since dashmap doesn't allow mutating keys safely iterations easily, we gather IDs to close
        let registry_len = self.registry.len();
        if registry_len > 0 {
            // Find positions in registry that are no longer in active_ids
            for entry in self.registry.positions.iter() {
                let pid = entry.key();
                if !active_ids.contains(pid) && entry.value().state != PositionState::Closed {
                    closed_ids.push(*pid);
                }
            }
        }

        for pid in closed_ids {
            if let Some(mut tracker) = self.registry.get(&pid) {
                info!("Detecting closed position: position_id={}", pid);
                tracker.state = PositionState::Closed;
                tracker.last_updated_at = SystemTime::now();
                tracker.realized_pnl = tracker.unrealized_pnl;
                tracker.unrealized_pnl = Decimal::ZERO;

                // Save closed state to DB and remove from registry
                if let Err(e) = self.store.save_position(&tracker).await {
                    error!("Failed to save closed position state to DB: {:?}", e);
                }
                self.registry.remove(&pid);
                info!(
                    "Position {} removed from cache and marked CLOSED in DB",
                    pid
                );
            }
        }

        Ok(())
    }
}

// Inline macro placeholder to simplify decimals
macro_rules! dec {
    ($val:expr) => {
        Decimal::from_f64($val).unwrap()
    };
}
use dec;

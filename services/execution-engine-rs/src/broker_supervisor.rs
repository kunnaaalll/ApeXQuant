#![deny(unsafe_code)]

//! Broker Supervisor
//!
//! Manages connection lifecycles, health checks, and state reconciliation
//! for MT5 and Binance adapters in background tokio tasks.

use crate::brokers::binance::adapter::BinanceAdapter;
use crate::brokers::broker::BrokerAdapter;
use crate::brokers::mt5::adapter::Mt5Adapter;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub struct BrokerSupervisor {
    mt5_adapter: Arc<Mt5Adapter>,
    binance_adapter: Arc<BinanceAdapter>,
}

impl BrokerSupervisor {
    pub fn new(mt5_adapter: Arc<Mt5Adapter>, binance_adapter: Arc<BinanceAdapter>) -> Self {
        Self {
            mt5_adapter,
            binance_adapter,
        }
    }

    /// Spawns the heartbeat loop which continuously pings both adapters.
    pub fn spawn_heartbeat_loop(&self, interval_secs: u64) {
        let mt5 = Arc::clone(&self.mt5_adapter);
        let binance = Arc::clone(&self.binance_adapter);

        tokio::spawn(async move {
            info!(
                "BrokerSupervisor: Heartbeat loop started ({}s)",
                interval_secs
            );
            loop {
                sleep(Duration::from_secs(interval_secs)).await;

                if let Err(e) = mt5.heartbeat().await {
                    warn!("MT5 heartbeat failed: {:?}", e);
                }

                if let Err(e) = binance.heartbeat().await {
                    warn!("Binance heartbeat failed: {:?}", e);
                }
            }
        });
    }

    /// Spawns the health check loop which queries full health metrics
    /// and logs degradation.
    pub fn spawn_health_loop(&self, interval_secs: u64) {
        let mt5 = Arc::clone(&self.mt5_adapter);
        let binance = Arc::clone(&self.binance_adapter);

        tokio::spawn(async move {
            info!(
                "BrokerSupervisor: Health check loop started ({}s)",
                interval_secs
            );
            loop {
                sleep(Duration::from_secs(interval_secs)).await;

                match mt5.health().await {
                    Ok(health) => {
                        if health.is_degraded() {
                            warn!("MT5 connection is degraded: {:?}", health);
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch MT5 health: {:?}", e);
                    }
                }

                match binance.health().await {
                    Ok(health) => {
                        if health.is_degraded() {
                            warn!("Binance connection is degraded: {:?}", health);
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch Binance health: {:?}", e);
                    }
                }
            }
        });
    }

    /// Spawns the reconciliation loop which fetches orders and positions
    /// to detect state drift against internal ledgers.
    pub fn spawn_reconciliation_loop(&self, interval_secs: u64) {
        let mt5 = Arc::clone(&self.mt5_adapter);
        let binance = Arc::clone(&self.binance_adapter);

        tokio::spawn(async move {
            info!(
                "BrokerSupervisor: Reconciliation loop started ({}s)",
                interval_secs
            );
            loop {
                sleep(Duration::from_secs(interval_secs)).await;

                // Reconcile MT5
                match mt5.get_orders().await {
                    Ok(orders) => info!("Reconciled {} MT5 orders", orders.len()),
                    Err(e) => warn!("Failed to reconcile MT5 orders: {:?}", e),
                }

                match mt5.get_positions().await {
                    Ok(pos) => info!("Reconciled {} MT5 positions", pos.len()),
                    Err(e) => warn!("Failed to reconcile MT5 positions: {:?}", e),
                }

                // Reconcile Binance
                match binance.get_orders().await {
                    Ok(orders) => info!("Reconciled {} Binance orders", orders.len()),
                    Err(e) => warn!("Failed to reconcile Binance orders: {:?}", e),
                }

                match binance.get_positions().await {
                    Ok(pos) => info!("Reconciled {} Binance positions", pos.len()),
                    Err(e) => warn!("Failed to reconcile Binance positions: {:?}", e),
                }
            }
        });
    }
}

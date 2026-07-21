// CPB-008: MT5 Stream — HTTP polling against mt5-bridge REST API
//
// Invariants:
//   - No unwrap / expect / panic
//   - Endpoint driven from Mt5Config (base URL, e.g. "http://mt5-bridge:8000")
//   - Polls /symbols/{symbol}/tick at configurable interval
//   - Exponential backoff on consecutive errors

use crate::streaming::TickStream;
use crate::tick::Tick;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

// ─── Configuration ────────────────────────────────────────────────────────────

/// All MT5 connectivity parameters.
#[derive(Debug, Clone)]
pub struct Mt5Config {
    /// Base HTTP URL of the mt5-bridge, e.g. "http://mt5-bridge:8000"
    pub endpoint: String,
    /// How often to poll for a new tick (milliseconds)
    pub poll_interval_ms: u64,
    /// Maximum consecutive errors before giving up (0 = unlimited)
    pub max_reconnect_attempts: u32,
    /// Base backoff delay in milliseconds (doubles each error, capped at 30 s)
    pub backoff_base_ms: u64,
}

impl Default for Mt5Config {
    fn default() -> Self {
        Self {
            endpoint: "http://mt5-bridge:8000".to_owned(),
            poll_interval_ms: 500,
            max_reconnect_attempts: 0, // unlimited — keep retrying forever
            backoff_base_ms: 250,
        }
    }
}

// ─── Feed Health ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedHealth {
    Healthy,
    Degraded,
    Reconnecting,
    Disconnected,
}

// ─── Stream Implementation ────────────────────────────────────────────────────

pub struct Mt5TickStream {
    symbol: String,
    config: Mt5Config,
    connected: bool,
    last_sequence: u64,
    last_timestamp: Option<DateTime<Utc>>,
    receiver: Option<mpsc::UnboundedReceiver<Tick>>,
    health: Arc<RwLock<FeedHealth>>,
    _task: Option<tokio::task::JoinHandle<()>>,
}

impl Mt5TickStream {
    pub fn new(symbol: String, config: Mt5Config) -> Self {
        Self {
            symbol,
            config,
            connected: false,
            last_sequence: 0,
            last_timestamp: None,
            receiver: None,
            health: Arc::new(RwLock::new(FeedHealth::Disconnected)),
            _task: None,
        }
    }

    pub fn health_handle(&self) -> Arc<RwLock<FeedHealth>> {
        self.health.clone()
    }
}

#[async_trait]
impl TickStream for Mt5TickStream {
    async fn connect(&mut self) -> Result<(), String> {
        let (tx, rx) = mpsc::unbounded_channel::<Tick>();
        self.receiver = Some(rx);

        let config = self.config.clone();
        let symbol = self.symbol.clone();
        let health = self.health.clone();

        let task = tokio::spawn(async move {
            run_http_poll_supervisor(symbol, config, tx, health).await;
        });

        self._task = Some(task);
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        if let Some(task) = self._task.take() {
            task.abort();
        }
        self.connected = false;
        self.receiver = None;
        let mut h = self.health.write().await;
        *h = FeedHealth::Disconnected;
        Ok(())
    }

    async fn next_tick(&mut self) -> Option<Tick> {
        if !self.connected {
            return None;
        }

        let mut tick = if let Some(rx) = &mut self.receiver {
            rx.recv().await?
        } else {
            return None;
        };

        // Assign monotonically-increasing sequence number
        self.last_sequence += 1;
        tick.sequence = self.last_sequence;
        self.last_timestamp = Some(tick.timestamp);

        Some(tick)
    }
}

// ─── HTTP Poll Supervisor ─────────────────────────────────────────────────────

async fn run_http_poll_supervisor(
    symbol: String,
    config: Mt5Config,
    tx: mpsc::UnboundedSender<Tick>,
    health: Arc<RwLock<FeedHealth>>,
) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(symbol = %symbol, error = %e, "MT5 HTTP: failed to build client");
            return;
        }
    };

    let url = format!("{}/symbols/{}/tick", config.endpoint, symbol);
    let max_attempts = config.max_reconnect_attempts;
    let mut consecutive_errors: u32 = 0;
    let mut last_timestamp: i64 = 0;

    tracing::info!(symbol = %symbol, url = %url, "MT5 HTTP: starting poll loop");

    {
        let mut h = health.write().await;
        *h = FeedHealth::Healthy;
    }

    loop {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        // Reset error counter on success
                        if consecutive_errors > 0 {
                            tracing::info!(symbol = %symbol, "MT5 HTTP: feed recovered");
                            let mut h = health.write().await;
                            *h = FeedHealth::Healthy;
                            consecutive_errors = 0;
                        }

                        // Parse fields — bridge returns: {symbol, bid, ask, last, time}
                        let ts = json["time"].as_i64().unwrap_or(0);
                        let bid_f = json["bid"].as_f64();
                        let ask_f = json["ask"].as_f64();

                        if let (Some(bid_f), Some(ask_f)) = (bid_f, ask_f) {
                            // Only emit if this is a new tick (timestamp changed)
                            if ts > last_timestamp && bid_f > 0.0 && ask_f > 0.0 {
                                last_timestamp = ts;

                                let bid_str = format!("{:.8}", bid_f);
                                let ask_str = format!("{:.8}", ask_f);

                                if let (Ok(bid), Ok(ask)) =
                                    (Decimal::from_str(&bid_str), Decimal::from_str(&ask_str))
                                {
                                    let timestamp = chrono::DateTime::from_timestamp(ts, 0)
                                        .unwrap_or_else(Utc::now);

                                    let tick = Tick {
                                        symbol: symbol.clone(),
                                        bid,
                                        ask,
                                        spread: ask - bid,
                                        timestamp,
                                        sequence: 0, // assigned by TickStream::next_tick
                                    };

                                    if tx.send(tick).is_err() {
                                        tracing::warn!(symbol = %symbol, "MT5 HTTP: receiver dropped, stopping poll");
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(symbol = %symbol, error = %e, "MT5 HTTP: failed to parse JSON");
                        consecutive_errors += 1;
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!(
                    symbol = %symbol,
                    status = %resp.status(),
                    "MT5 HTTP: non-200 response"
                );
                consecutive_errors += 1;
            }
            Err(e) => {
                tracing::warn!(symbol = %symbol, error = %e, attempt = consecutive_errors, "MT5 HTTP: request failed");
                consecutive_errors += 1;
            }
        }

        // Check if we've exceeded max attempts
        if max_attempts > 0 && consecutive_errors >= max_attempts {
            tracing::error!(
                symbol = %symbol,
                attempts = consecutive_errors,
                "MT5 HTTP: max errors reached — feed permanently disconnected"
            );
            let mut h = health.write().await;
            *h = FeedHealth::Disconnected;
            return;
        }

        // If errors are accumulating, mark degraded and back off
        if consecutive_errors > 0 {
            let mut h = health.write().await;
            *h = FeedHealth::Degraded;
            let delay = exponential_backoff_ms(consecutive_errors, config.backoff_base_ms);
            time::sleep(Duration::from_millis(delay)).await;
        } else {
            // Normal poll interval
            time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn exponential_backoff_ms(attempt: u32, base_ms: u64) -> u64 {
    let exp = (2u64).saturating_pow(attempt.saturating_sub(1));
    base_ms.saturating_mul(exp).min(30_000)
}

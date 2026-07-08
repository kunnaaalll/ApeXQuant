use crate::tick::Tick;
use crate::streaming::TickStream;
use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedHealth {
    Healthy,
    Degraded,
    Reconnecting,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct BinanceConfig {
    pub endpoint: String,
    pub max_reconnect_attempts: u32,
    pub backoff_base_ms: u64,
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            endpoint: "wss://stream.binance.com:9443/ws/".to_owned(),
            max_reconnect_attempts: 10,
            backoff_base_ms: 250,
        }
    }
}

pub struct BinanceTickStream {
    symbol: String,
    config: BinanceConfig,
    connected: bool,
    last_sequence: u64,
    receiver: Option<mpsc::UnboundedReceiver<Tick>>,
    health: Arc<RwLock<FeedHealth>>,
    _task: Option<tokio::task::JoinHandle<()>>,
}

impl BinanceTickStream {
    pub fn new_spot(symbol: String) -> Self {
        let mut config = BinanceConfig::default();
        config.endpoint = format!("wss://stream.binance.com:9443/ws/{}@bookTicker", symbol.to_lowercase());
        Self::new(symbol, config)
    }

    pub fn new_futures(symbol: String) -> Self {
        let mut config = BinanceConfig::default();
        config.endpoint = format!("wss://fstream.binance.com/ws/{}@bookTicker", symbol.to_lowercase());
        Self::new(symbol, config)
    }

    pub fn new(symbol: String, config: BinanceConfig) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            config,
            connected: false,
            last_sequence: 0,
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
impl TickStream for BinanceTickStream {
    async fn connect(&mut self) -> Result<(), String> {
        let (tx, rx) = mpsc::unbounded_channel::<Tick>();
        self.receiver = Some(rx);

        let config = self.config.clone();
        let symbol = self.symbol.clone();
        let health = self.health.clone();

        let task = tokio::spawn(async move {
            run_reconnect_supervisor(symbol, config, tx, health).await;
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

        self.last_sequence += 1;
        tick.sequence = self.last_sequence;

        Some(tick)
    }
}

async fn run_reconnect_supervisor(
    symbol: String,
    config: BinanceConfig,
    tx: mpsc::UnboundedSender<Tick>,
    health: Arc<RwLock<FeedHealth>>,
) {
    let max_attempts = config.max_reconnect_attempts;
    let mut attempt = 0u32;

    loop {
        if max_attempts > 0 && attempt >= max_attempts {
            tracing::error!(symbol = %symbol, attempts = attempt, "Binance: max reconnect attempts reached");
            let mut h = health.write().await;
            *h = FeedHealth::Disconnected;
            return;
        }

        {
            let mut h = health.write().await;
            *h = if attempt == 0 { FeedHealth::Healthy } else { FeedHealth::Reconnecting };
        }

        tracing::info!(symbol = %symbol, attempt, endpoint = %config.endpoint, "Binance: connecting");

        match try_connect_and_stream(&symbol, &config, &tx, &health).await {
            Ok(()) => {
                tracing::info!(symbol = %symbol, "Binance: stream ended cleanly");
                attempt = 0;
            }
            Err(e) => {
                tracing::warn!(symbol = %symbol, attempt, error = %e, "Binance: connection error");
                attempt += 1;
            }
        }

        let delay_ms = exponential_backoff_ms(attempt, config.backoff_base_ms);
        {
            let mut h = health.write().await;
            *h = FeedHealth::Reconnecting;
        }
        time::sleep(Duration::from_millis(delay_ms)).await;
    }
}

async fn try_connect_and_stream(
    symbol: &str,
    config: &BinanceConfig,
    tx: &mpsc::UnboundedSender<Tick>,
    health: &Arc<RwLock<FeedHealth>>,
) -> Result<(), String> {
    let (ws_stream, _) = connect_async(&config.endpoint)
        .await
        .map_err(|e| format!("WS connect to {} failed: {}", config.endpoint, e))?;

    {
        let mut h = health.write().await;
        *h = FeedHealth::Healthy;
    }

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let (Some(b), Some(a)) = (json["b"].as_str(), json["a"].as_str()) {
                        if let (Ok(bid), Ok(ask)) = (Decimal::from_str(b), Decimal::from_str(a)) {
                            // Extract timestamp E if present (Futures), otherwise use Utc::now() (Spot)
                            let timestamp = json["E"]
                                .as_i64()
                                .map(|t| chrono::DateTime::from_timestamp_millis(t).unwrap_or_else(Utc::now))
                                .unwrap_or_else(Utc::now);

                            let spread = ask - bid;
                            if spread >= Decimal::ZERO {
                                let tick = Tick {
                                    symbol: symbol.to_owned(),
                                    bid,
                                    ask,
                                    spread,
                                    timestamp,
                                    sequence: 0,
                                };
                                if tx.send(tick).is_err() {
                                    return Err("tick receiver dropped".to_owned());
                                }
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(e) => {
                return Err(format!("WS read error: {}", e));
            }
            _ => {}
        }
    }

    Ok(())
}

fn exponential_backoff_ms(attempt: u32, base_ms: u64) -> u64 {
    let exp = (2u64).saturating_pow(attempt.saturating_sub(1));
    base_ms.saturating_mul(exp).min(30_000)
}

// CPB-008: MT5 Stream — configurable endpoint, auth, heartbeat, reconnect supervisor
//
// Invariants:
//   - No unwrap / expect / panic
//   - Endpoint and credentials driven entirely from Mt5Config
//   - Exponential backoff capped at max_reconnect_attempts
//   - FeedHealth reported through shared Arc<RwLock<FeedHealth>>

use crate::streaming::TickStream;
use crate::tick::Tick;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ring::hmac;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

// ─── Configuration ────────────────────────────────────────────────────────────

/// All MT5 connectivity parameters — no values hardcoded in logic.
#[derive(Debug, Clone)]
pub struct Mt5Config {
    /// TCP endpoint, e.g. "127.0.0.1:5555" or "bridge.prod.apex:5555"
    pub endpoint: String,
    /// HMAC-SHA256 auth token (raw bytes); empty slice = no auth
    pub auth_token: Vec<u8>,
    /// How often to send a heartbeat ping (milliseconds)
    pub heartbeat_interval_ms: u64,
    /// Maximum reconnect attempts before giving up (0 = unlimited)
    pub max_reconnect_attempts: u32,
    /// Base backoff delay in milliseconds (doubles each attempt, capped at 30 s)
    pub backoff_base_ms: u64,
}

impl Default for Mt5Config {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:5555".to_owned(),
            auth_token: Vec::new(),
            heartbeat_interval_ms: 5_000,
            max_reconnect_attempts: 10,
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

    /// Expose the health handle so external supervisors can poll it.
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

        // Assign monotonically-increasing sequence number
        self.last_sequence += 1;
        tick.sequence = self.last_sequence;

        // Record last timestamp for ordering diagnostics
        self.last_timestamp = Some(tick.timestamp);

        Some(tick)
    }
}

// ─── Reconnect Supervisor ─────────────────────────────────────────────────────

/// Manages the connection lifecycle with exponential backoff.
/// Runs indefinitely (or until max_reconnect_attempts is exceeded).
async fn run_reconnect_supervisor(
    symbol: String,
    config: Mt5Config,
    tx: mpsc::UnboundedSender<Tick>,
    health: Arc<RwLock<FeedHealth>>,
) {
    let max_attempts = config.max_reconnect_attempts;
    let mut attempt = 0u32;

    loop {
        if max_attempts > 0 && attempt >= max_attempts {
            tracing::error!(
                symbol = %symbol,
                attempts = attempt,
                "MT5: max reconnect attempts reached — feed permanently disconnected"
            );
            let mut h = health.write().await;
            *h = FeedHealth::Disconnected;
            return;
        }

        {
            let mut h = health.write().await;
            *h = if attempt == 0 {
                FeedHealth::Healthy
            } else {
                FeedHealth::Reconnecting
            };
        }

        tracing::info!(symbol = %symbol, attempt, endpoint = %config.endpoint, "MT5: connecting");

        match try_connect_and_stream(&symbol, &config, &tx, &health).await {
            Ok(()) => {
                // Clean disconnect (stream ended); reset backoff
                tracing::info!(symbol = %symbol, "MT5: stream ended cleanly — reconnecting");
                attempt = 0;
            }
            Err(e) => {
                tracing::warn!(symbol = %symbol, attempt, error = %e, "MT5: connection error");
                attempt += 1;
            }
        }

        let delay_ms = exponential_backoff_ms(attempt, config.backoff_base_ms);
        {
            let mut h = health.write().await;
            *h = FeedHealth::Reconnecting;
        }
        tracing::debug!(symbol = %symbol, delay_ms, "MT5: backing off before reconnect");
        time::sleep(Duration::from_millis(delay_ms)).await;
    }
}

/// Attempt a single TCP connection, authenticate, then stream ticks until error.
async fn try_connect_and_stream(
    symbol: &str,
    config: &Mt5Config,
    tx: &mpsc::UnboundedSender<Tick>,
    health: &Arc<RwLock<FeedHealth>>,
) -> Result<(), String> {
    // ── Connect ──────────────────────────────────────────────────────────────
    let mut stream = TcpStream::connect(&config.endpoint)
        .await
        .map_err(|e| format!("TCP connect to {} failed: {e}", config.endpoint))?;

    // ── Authenticate ─────────────────────────────────────────────────────────
    if !config.auth_token.is_empty() {
        let challenge = format!("AUTH:{symbol}:{}", Utc::now().timestamp_millis());
        let key = hmac::Key::new(hmac::HMAC_SHA256, &config.auth_token);
        let sig = hmac::sign(&key, challenge.as_bytes());
        let hex_sig = hex_encode(sig.as_ref());
        let auth_line = format!("{challenge}:{hex_sig}\n");

        stream
            .write_all(auth_line.as_bytes())
            .await
            .map_err(|e| format!("auth write failed: {e}"))?;
    }

    // Mark healthy after successful auth
    {
        let mut h = health.write().await;
        *h = FeedHealth::Healthy;
    }

    let heartbeat_ms = config.heartbeat_interval_ms;
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    // ── Heartbeat task ───────────────────────────────────────────────────────
    let hb_health = health.clone();
    let hb_symbol = symbol.to_owned();
    let heartbeat_task = tokio::spawn(async move {
        let interval = Duration::from_millis(heartbeat_ms);
        loop {
            time::sleep(interval).await;
            let ping = format!("PING:{hb_symbol}\n");
            if write_half.write_all(ping.as_bytes()).await.is_err() {
                tracing::warn!(symbol = %hb_symbol, "MT5: heartbeat write failed");
                let mut h = hb_health.write().await;
                *h = FeedHealth::Degraded;
                break;
            }
        }
    });

    // ── Tick reader loop ─────────────────────────────────────────────────────
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF / clean close
            Err(e) => {
                heartbeat_task.abort();
                return Err(format!("read error: {e}"));
            }
            Ok(_) => {}
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("PONG") {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if json["symbol"].as_str() != Some(symbol) {
                continue;
            }

            if let (Some(b), Some(a), Some(t)) = (
                json["bid"].as_str(),
                json["ask"].as_str(),
                json["timestamp"].as_i64(),
            ) {
                if let (Ok(bid), Ok(ask)) = (Decimal::from_str(b), Decimal::from_str(a)) {
                    let timestamp =
                        chrono::DateTime::from_timestamp_millis(t).unwrap_or_else(Utc::now);
                    let tick = Tick {
                        symbol: symbol.to_owned(),
                        bid,
                        ask,
                        spread: ask - bid,
                        timestamp,
                        sequence: 0, // assigned by TickStream::next_tick
                    };
                    if tx.send(tick).is_err() {
                        // Receiver dropped — supervisor will restart
                        heartbeat_task.abort();
                        return Err("tick receiver dropped".to_owned());
                    }
                }
            }
        }
    }

    heartbeat_task.abort();
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Exponential backoff capped at 30 000 ms.
fn exponential_backoff_ms(attempt: u32, base_ms: u64) -> u64 {
    let exp = (2u64).saturating_pow(attempt.saturating_sub(1));
    base_ms.saturating_mul(exp).min(30_000)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
        s
    })
}

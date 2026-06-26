use crate::brokers::broker::BrokerAdapter;
use crate::brokers::errors::BrokerError;
use crate::brokers::health::BrokerHealth;
use crate::brokers::requests::{ClosePositionRequest, OrderCancelRequest, OrderModifyRequest, OrderSubmitRequest};
use crate::brokers::responses::{
    AccountInfo, ClosePositionResponse, OpenPosition, OrderCancelResponse, OrderModifyResponse,
    OrderSubmitResponse, PendingOrder, SymbolInfo,
};
use crate::brokers::binance::account::BinanceAccount;
use crate::brokers::binance::orders::BinanceOrder;
use crate::brokers::binance::positions::BinancePosition;
use crate::brokers::binance::symbols::BinanceSymbol;
use crate::brokers::connection::ConnectionState;
use crate::brokers::capabilities::BrokerCapabilities;

use async_trait::async_trait;
use ring::hmac;
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

pub struct BinanceAdapter {
    pub broker_id: String,
    http_client: reqwest::Client,
    base_url: String,
    api_key: String,
    secret_key: hmac::Key,
    connection_state: Arc<RwLock<ConnectionState>>,
    connect_time: Arc<RwLock<Option<SystemTime>>>,
    reconnect_count: Arc<AtomicU32>,
}

impl BinanceAdapter {
    pub fn new(broker_id: String, base_url: String, api_key: String, secret: String) -> Self {
        let secret_key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
        Self {
            broker_id,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            base_url,
            api_key,
            secret_key,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            connect_time: Arc::new(RwLock::new(None)),
            reconnect_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    fn sign(&self, query: &str) -> String {
        let tag = hmac::sign(&self.secret_key, query.as_bytes());
        hex::encode(tag.as_ref())
    }

    pub async fn connect(&self) -> Result<(), BrokerError> {
        let mut state = self.connection_state.write().await;
        if *state == ConnectionState::Connected {
            return Ok(());
        }

        let url = format!("{}/fapi/v1/ping", self.base_url);
        let res = self.http_client.get(&url).send().await.map_err(|e| {
            *state = ConnectionState::Failed;
            BrokerError::ConnectionFailure(e.to_string())
        })?;

        if res.status().is_success() {
            *state = ConnectionState::Connected;
            let mut ct = self.connect_time.write().await;
            *ct = Some(SystemTime::now());
            Ok(())
        } else {
            *state = ConnectionState::Failed;
            Err(BrokerError::ConnectionFailure(format!("HTTP {}", res.status())))
        }
    }

    pub async fn disconnect(&self) -> Result<(), BrokerError> {
        let mut state = self.connection_state.write().await;
        *state = ConnectionState::Disconnected;
        let mut ct = self.connect_time.write().await;
        *ct = None;
        Ok(())
    }

    pub async fn heartbeat(&self) -> Result<(), BrokerError> {
        self.ping().await
    }

    pub async fn reconnect(&self) -> Result<(), BrokerError> {
        self.reconnect_count.fetch_add(1, Ordering::SeqCst);
        let _ = self.disconnect().await;
        self.connect().await
    }

    pub async fn get_capabilities(&self) -> BrokerCapabilities {
        BrokerCapabilities {
            supports_hedging: false,
            supports_crypto: true,
            supports_forex: false,
            supports_fractional_lots: true,
        }
    }

    pub async fn shutdown(&self) -> Result<(), BrokerError> {
        self.disconnect().await
    }
}

#[async_trait]
impl BrokerAdapter for BinanceAdapter {
    async fn get_account(&self) -> Result<AccountInfo, BrokerError> {
        let qs = format!("timestamp={}", Self::timestamp());
        let signature = self.sign(&qs);
        let url = format!("{}/fapi/v2/account?{}&signature={}", self.base_url, qs, signature);

        let res = self.http_client.get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send().await.map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::AccountError(format!("HTTP {}", res.status())));
        }

        let account: BinanceAccount = res.json().await.map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;
        Ok(account.into())
    }

    async fn get_symbol(&self, symbol: &str) -> Result<SymbolInfo, BrokerError> {
        let url = format!("{}/fapi/v1/exchangeInfo", self.base_url);
        let res = self.http_client.get(&url).send().await.map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::InternalError(format!("HTTP {}", res.status())));
        }

        #[derive(serde::Deserialize)]
        struct ExchangeInfo {
            symbols: Vec<BinanceSymbol>,
        }

        let info: ExchangeInfo = res.json().await.map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        let sym = info.symbols.into_iter().find(|s| s.symbol == symbol)
            .ok_or_else(|| BrokerError::SymbolNotFound(symbol.to_string()))?;

        Ok(sym.into())
    }

    async fn get_positions(&self) -> Result<Vec<OpenPosition>, BrokerError> {
        let qs = format!("timestamp={}", Self::timestamp());
        let signature = self.sign(&qs);
        let url = format!("{}/fapi/v2/positionRisk?{}&signature={}", self.base_url, qs, signature);

        let res = self.http_client.get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send().await.map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::InternalError(format!("HTTP {}", res.status())));
        }

        let positions: Vec<BinancePosition> = res.json().await.map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(positions.into_iter()
            .filter(|p| !p.position_amount.is_zero())
            .map(|p| p.into())
            .collect())
    }

    async fn get_orders(&self) -> Result<Vec<PendingOrder>, BrokerError> {
        let qs = format!("timestamp={}", Self::timestamp());
        let signature = self.sign(&qs);
        let url = format!("{}/fapi/v1/openOrders?{}&signature={}", self.base_url, qs, signature);

        let res = self.http_client.get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send().await.map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::InternalError(format!("HTTP {}", res.status())));
        }

        let orders: Vec<BinanceOrder> = res.json().await.map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(orders.into_iter().map(|o| o.into()).collect())
    }

    async fn submit_order(&self, req: OrderSubmitRequest) -> Result<OrderSubmitResponse, BrokerError> {
        let side = match req.side {
            crate::brokers::requests::OrderSide::Buy => "BUY",
            crate::brokers::requests::OrderSide::Sell => "SELL",
        };
        let r#type = match req.order_type {
            crate::brokers::requests::OrderType::Market => "MARKET",
            crate::brokers::requests::OrderType::Limit => "LIMIT",
            crate::brokers::requests::OrderType::Stop => "STOP_MARKET",
            crate::brokers::requests::OrderType::StopLimit => "STOP",
        };

        let mut qs = format!("symbol={}&side={}&type={}&quantity={}&timestamp={}",
                             req.symbol, side, r#type, req.volume, Self::timestamp());
        
        if let Some(p) = req.price {
            qs.push_str(&format!("&price={}&timeInForce=GTC", p));
        }
        if let Some(sl) = req.stop_loss {
            qs.push_str(&format!("&stopPrice={}", sl));
        }

        let signature = self.sign(&qs);
        let url = format!("{}/fapi/v1/order?{}&signature={}", self.base_url, qs, signature);

        let res = self.http_client.post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send().await.map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::OrderSubmissionFailed(format!("HTTP {}", res.status())));
        }

        #[derive(serde::Deserialize)]
        struct SubmitResp {
            #[serde(rename = "orderId")]
            order_id: i64,
        }

        let resp: SubmitResp = res.json().await.map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(OrderSubmitResponse {
            order_id: resp.order_id.to_string(),
        })
    }

    async fn modify_order(&self, _req: OrderModifyRequest) -> Result<OrderModifyResponse, BrokerError> {
        Err(BrokerError::OrderModificationFailed("Binance Futures does not support modify via REST, requires cancel/replace".to_string()))
    }

    async fn cancel_order(&self, _req: OrderCancelRequest) -> Result<OrderCancelResponse, BrokerError> {
        // We need the symbol for Binance cancel order which is not in OrderCancelRequest directly.
        // As a workaround for this signature we might need to change it or pass dummy symbol.
        // Assuming we store symbol mapped to order_id, or we modify the trait if allowed.
        // The instructions said "No new traits unless absolutely required by an existing interface" and we shouldn't modify the interface either.
        // For Binance we HAVE to pass symbol. This is a known issue with Binance API.
        Err(BrokerError::OrderCancellationFailed("Binance cancel order requires symbol parameter which is missing in OrderCancelRequest".to_string()))
    }

    async fn close_position(&self, _req: ClosePositionRequest) -> Result<ClosePositionResponse, BrokerError> {
        // Similar issue, need symbol.
        Err(BrokerError::PositionCloseFailed("Binance requires symbol to close position".to_string()))
    }

    async fn health(&self) -> Result<BrokerHealth, BrokerError> {
        let start = SystemTime::now();
        self.ping().await?;
        let latency_us = start.elapsed().unwrap_or_default().as_micros() as u64;
        let latency_ms = Decimal::from(latency_us) / rust_decimal_macros::dec!(1000.0);

        let ct_guard = self.connect_time.read().await;
        let is_up = ct_guard.is_some();
        let uptime_percentage = if is_up {
            rust_decimal_macros::dec!(100.0)
        } else {
            rust_decimal_macros::dec!(0.0)
        };

        Ok(BrokerHealth {
            latency_ms,
            uptime_percentage,
            heartbeat_interval_ms: rust_decimal_macros::dec!(30_000.0),
            last_response_time: SystemTime::now(),
            reconnect_attempts: self.reconnect_count.load(Ordering::SeqCst),
        })
    }

    async fn ping(&self) -> Result<(), BrokerError> {
        let url = format!("{}/fapi/v1/ping", self.base_url);
        let res = self.http_client.get(&url).send().await.map_err(|e| {
            let mut state = futures::executor::block_on(self.connection_state.write());
            *state = ConnectionState::Failed;
            BrokerError::ConnectionFailure(e.to_string())
        })?;

        if res.status().is_success() {
            let mut state = self.connection_state.write().await;
            if *state == ConnectionState::Failed || *state == ConnectionState::Disconnected {
                *state = ConnectionState::Connected;
            }
            Ok(())
        } else {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Failed;
            Err(BrokerError::ConnectionFailure(format!("HTTP {}", res.status())))
        }
    }
}

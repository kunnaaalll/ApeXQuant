use crate::brokers::broker::BrokerAdapter;
use crate::brokers::capabilities::BrokerCapabilities;
use crate::brokers::connection::ConnectionState;
use crate::brokers::errors::BrokerError;
use crate::brokers::health::BrokerHealth;
use crate::brokers::mt5::account::Mt5Account;
use crate::brokers::mt5::orders::Mt5Order;
use crate::brokers::mt5::positions::Mt5Position;
use crate::brokers::mt5::symbols::Mt5Symbol;
use crate::brokers::requests::{
    ClosePositionRequest, OrderCancelRequest, OrderModifyRequest, OrderSubmitRequest,
};
use crate::brokers::responses::{
    AccountInfo, ClosePositionResponse, OpenPosition, OrderCancelResponse, OrderModifyResponse,
    OrderSubmitResponse, PendingOrder, SymbolInfo,
};

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

pub struct Mt5Adapter {
    pub broker_id: String,
    http_client: reqwest::Client,
    bridge_url: String,
    connection_state: Arc<RwLock<ConnectionState>>,
    connect_time: Arc<RwLock<Option<SystemTime>>>,
    reconnect_count: Arc<AtomicU32>,
}

impl Mt5Adapter {
    pub fn new(broker_id: String, bridge_url: String) -> Self {
        Self {
            broker_id,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            bridge_url,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            connect_time: Arc::new(RwLock::new(None)),
            reconnect_count: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn connect(&self) -> Result<(), BrokerError> {
        {
            let state = self.connection_state.read().await;
            if *state == ConnectionState::Connected {
                return Ok(());
            }
        }

        // Attempt connection to bridge
        let url = format!("{}/connect", self.bridge_url);
        let res = match self.http_client.post(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                let mut state = self.connection_state.write().await;
                *state = ConnectionState::Failed;
                return Err(BrokerError::ConnectionFailure(e.to_string()));
            }
        };

        let mut state = self.connection_state.write().await;
        if res.status().is_success() {
            *state = ConnectionState::Connected;
            let mut ct = self.connect_time.write().await;
            *ct = Some(SystemTime::now());
            Ok(())
        } else {
            *state = ConnectionState::Failed;
            Err(BrokerError::ConnectionFailure(format!(
                "HTTP {}",
                res.status()
            )))
        }
    }

    pub async fn disconnect(&self) -> Result<(), BrokerError> {
        let mut state = self.connection_state.write().await;
        let url = format!("{}/disconnect", self.bridge_url);
        let _ = self.http_client.post(&url).send().await; // Ignore failure on disconnect
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
            supports_hedging: true,
            supports_crypto: true,
            supports_forex: true,
            supports_fractional_lots: true,
        }
    }

    pub async fn shutdown(&self) -> Result<(), BrokerError> {
        self.disconnect().await
    }
}

#[async_trait]
impl BrokerAdapter for Mt5Adapter {
    async fn get_account(&self) -> Result<AccountInfo, BrokerError> {
        let url = format!("{}/account", self.bridge_url);
        let res = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::AccountError(format!("HTTP {}", res.status())));
        }

        let account: Mt5Account = res
            .json()
            .await
            .map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(account.into())
    }

    async fn get_symbol(&self, symbol: &str) -> Result<SymbolInfo, BrokerError> {
        let url = format!("{}/symbols/{}", self.bridge_url, symbol);
        let res = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if res.status().as_u16() == 404 {
            return Err(BrokerError::SymbolNotFound(symbol.to_string()));
        } else if !res.status().is_success() {
            return Err(BrokerError::InternalError(format!("HTTP {}", res.status())));
        }

        let sym: Mt5Symbol = res
            .json()
            .await
            .map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(sym.into())
    }

    async fn get_positions(&self) -> Result<Vec<OpenPosition>, BrokerError> {
        let url = format!("{}/positions", self.bridge_url);
        let res = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::InternalError(format!("HTTP {}", res.status())));
        }

        let positions: Vec<Mt5Position> = res
            .json()
            .await
            .map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(positions.into_iter().map(|p| p.into()).collect())
    }

    async fn get_orders(&self) -> Result<Vec<PendingOrder>, BrokerError> {
        let url = format!("{}/orders", self.bridge_url);
        let res = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::InternalError(format!("HTTP {}", res.status())));
        }

        let orders: Vec<Mt5Order> = res
            .json()
            .await
            .map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(orders.into_iter().map(|o| o.into()).collect())
    }

    async fn submit_order(
        &self,
        req: OrderSubmitRequest,
    ) -> Result<OrderSubmitResponse, BrokerError> {
        let url = format!("{}/orders", self.bridge_url);
        let res = self
            .http_client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::OrderSubmissionFailed(format!(
                "HTTP {}",
                res.status()
            )));
        }

        #[derive(serde::Deserialize)]
        struct SubmitResp {
            order_id: String,
        }

        let resp: SubmitResp = res
            .json()
            .await
            .map_err(|e| BrokerError::InvalidMessage(e.to_string()))?;

        Ok(OrderSubmitResponse {
            order_id: resp.order_id,
        })
    }

    async fn modify_order(
        &self,
        req: OrderModifyRequest,
    ) -> Result<OrderModifyResponse, BrokerError> {
        let url = format!("{}/orders/{}", self.bridge_url, req.order_id);
        let res = self
            .http_client
            .put(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::OrderModificationFailed(format!(
                "HTTP {}",
                res.status()
            )));
        }

        Ok(OrderModifyResponse { success: true })
    }

    async fn cancel_order(
        &self,
        req: OrderCancelRequest,
    ) -> Result<OrderCancelResponse, BrokerError> {
        let url = format!("{}/orders/{}", self.bridge_url, req.order_id);
        let res = self
            .http_client
            .delete(&url)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::OrderCancellationFailed(format!(
                "HTTP {}",
                res.status()
            )));
        }

        Ok(OrderCancelResponse { success: true })
    }

    async fn close_position(
        &self,
        req: ClosePositionRequest,
    ) -> Result<ClosePositionResponse, BrokerError> {
        let url = format!("{}/positions/{}/close", self.bridge_url, req.position_id);
        let res = self
            .http_client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| BrokerError::ConnectionFailure(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BrokerError::PositionCloseFailed(format!(
                "HTTP {}",
                res.status()
            )));
        }

        Ok(ClosePositionResponse { success: true })
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
        let url = format!("{}/ping", self.bridge_url);
        let res = match self.http_client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                let mut state = self.connection_state.write().await;
                *state = ConnectionState::Failed;
                return Err(BrokerError::ConnectionFailure(e.to_string()));
            }
        };

        if res.status().is_success() {
            let mut state = self.connection_state.write().await;
            if *state == ConnectionState::Failed || *state == ConnectionState::Disconnected {
                *state = ConnectionState::Connected;
            }
            Ok(())
        } else {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Failed;
            Err(BrokerError::ConnectionFailure(format!(
                "HTTP {}",
                res.status()
            )))
        }
    }
}

use apex_protos::events::{Event, SubscribeRequest, event_bus_service_client::EventBusServiceClient, AckRequest, AckResponse, event::Payload};
use anyhow::{Result, Context};
use tokio::sync::mpsc;
use tonic::transport::Channel;
use tracing::{info, warn, error};
use std::sync::Arc;
use rust_decimal::Decimal;
use uuid::Uuid;
use sqlx::{PgPool, Row};

use crate::portfolio::registry::PortfolioRegistry;
use crate::exposure::registry::ExposureRegistry;
use crate::portfolio::events::PortfolioEvent;
use crate::exposure::events::ExposureEvent;
use crate::exposure::sector::Sector;
use crate::exposure::currency::Currency;
use crate::event_bus::EventBusPublisher;

#[derive(Clone)]
pub struct EventBusSubscriber {
    client: EventBusServiceClient<Channel>,
    consumer_group: String,
    consumer_id: String,
}

impl EventBusSubscriber {
    pub async fn connect(url: String, consumer_group: String, consumer_id: String) -> Result<Self> {
        let client = EventBusServiceClient::connect(url).await
            .context("Failed to connect to EventBusService")?;
        Ok(Self { client, consumer_group, consumer_id })
    }

    pub async fn subscribe(
        &self, 
        topic: &str, 
    ) -> Result<mpsc::Receiver<Event>> {
        let (tx, rx) = mpsc::channel(100);
        
        let req = SubscribeRequest {
            topics: vec![topic.to_string()],
            consumer_id: self.consumer_id.clone(),
            consumer_group: self.consumer_group.clone(),
            start_from: None,
            max_batch_size: 100,
            max_wait_ms: Some(apex_protos::common::Duration {
                seconds: 0,
                nanos: 50_000_000,
            }),
            filter: None,
        };

        let mut client = self.client.clone();
        let request = tonic::Request::new(req);
        let mut stream = client.subscribe(request).await?.into_inner();

        tokio::spawn(async move {
            while let Ok(Some(batch)) = stream.message().await {
                for event in batch.events {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    pub async fn ack(&self, event_ids: Vec<String>) -> Result<AckResponse> {
        let mut client = self.client.clone();
        let req = AckRequest {
            consumer_group: self.consumer_group.clone(),
            consumer_id: self.consumer_id.clone(),
            event_ids,
            failed: vec![],
        };
        
        let response = client.ack(tonic::Request::new(req)).await
            .context("Failed to ack events")?;
            
        Ok(response.into_inner())
    }

    pub async fn start_listening(
        &self,
        mut rx: mpsc::Receiver<Event>,
        portfolio_registry: PortfolioRegistry,
        exposure_registry: ExposureRegistry,
        pool: PgPool,
        _publisher: Option<Arc<EventBusPublisher>>,
    ) {
        let subscriber_self = self.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let event_id_str = event.event_id.as_ref().map(|id| uuid::Uuid::from_slice(&id.value).map(|u| u.to_string()).unwrap_or_default()).unwrap_or_default();
                
                if let Some(payload) = &event.payload {
                    match payload {
                        Payload::PositionOpened(po) => {
                            info!("Processing PositionOpenedEvent for position: {}", po.position_id);
                            if let Err(e) = handle_position_opened(po, &portfolio_registry, &exposure_registry, &pool).await {
                                error!("Error handling position opened: {:?}", e);
                            }
                        }
                        Payload::PositionClosed(pc) => {
                            info!("Processing PositionClosedEvent for position: {}", pc.position_id);
                            if let Err(e) = handle_position_closed(pc, &portfolio_registry, &exposure_registry, &pool).await {
                                error!("Error handling position closed: {:?}", e);
                            }
                        }
                        Payload::TickReceived(tick) => {
                            if let Err(e) = handle_tick_received(tick, &portfolio_registry, &exposure_registry, &pool).await {
                                error!("Error handling tick received: {:?}", e);
                            }
                        }
                        Payload::DrawdownLimitReached(dd) => {
                            warn!("Drawdown limit reached event received: {:?}", dd);
                            let _ = portfolio_registry.dispatch(PortfolioEvent::RecoveryTransition {
                                new_state: crate::portfolio::state::RecoveryState::Warning,
                                reason: "Drawdown limit reached".to_string(),
                            });
                        }
                        _ => {
                            // Other events we don't consume for state calculation
                        }
                    }
                }

                // Auto-acknowledge event
                if !event_id_str.is_empty() {
                    let _ = subscriber_self.ack(vec![event_id_str]).await;
                }
            }
        });
    }
}

// Helpers

fn parse_sector(symbol: &str) -> Sector {
    let sym_upper = symbol.to_uppercase();
    if sym_upper.contains("BTC") || sym_upper.contains("ETH") || sym_upper.contains("SOL") {
        Sector::Crypto
    } else if sym_upper.contains("XAU") || sym_upper.contains("XAG") || sym_upper.contains("GOLD") {
        Sector::Metals
    } else if sym_upper.contains("NAS") || sym_upper.contains("SPX") || sym_upper.contains("DJI") || sym_upper.contains("US30") {
        Sector::Indices
    } else if sym_upper.contains("OIL") || sym_upper.contains("GAS") {
        Sector::Commodities
    } else if sym_upper.contains("US10Y") {
        Sector::Bonds
    } else {
        Sector::Forex
    }
}

fn parse_currencies(symbol: &str) -> (Currency, Currency) {
    let sym_upper = symbol.to_uppercase();
    let base = if sym_upper.starts_with("EUR") {
        Currency::EUR
    } else if sym_upper.starts_with("GBP") {
        Currency::GBP
    } else if sym_upper.starts_with("USD") {
        Currency::USD
    } else if sym_upper.starts_with("JPY") {
        Currency::JPY
    } else if sym_upper.starts_with("XAU") {
        Currency::XAU
    } else if sym_upper.starts_with("BTC") {
        Currency::BTC
    } else {
        Currency::EUR
    };

    let quote = if sym_upper.ends_with("USD") {
        Currency::USD
    } else if sym_upper.ends_with("JPY") {
        Currency::JPY
    } else if sym_upper.ends_with("EUR") {
        Currency::EUR
    } else if sym_upper.ends_with("GBP") {
        Currency::GBP
    } else {
        Currency::USD
    };

    (base, quote)
}

async fn handle_position_opened(
    po: &apex_protos::events::PositionOpenedEvent,
    portfolio: &PortfolioRegistry,
    exposure: &ExposureRegistry,
    pool: &PgPool,
) -> Result<()> {
    let position_uuid = Uuid::parse_str(&po.position_id).context("Invalid position UUID")?;
    let entry_price = po.entry_price.as_ref()
        .map(|p| p.value.parse::<Decimal>().unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);
    let volume = po.initial_volume.as_ref()
        .map(|v| v.units.parse::<Decimal>().unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);

    let contract_size = if po.symbol.contains("BTC") || po.symbol.contains("ETH") || po.symbol.contains("XAU") {
        Decimal::ONE
    } else {
        Decimal::from(100_000)
    };

    let calculated_exposure = volume * entry_price * contract_size;
    let margin_used = calculated_exposure / Decimal::from(100);

    portfolio.dispatch(PortfolioEvent::PositionOpened {
        position_id: position_uuid,
        margin_used,
        exposure: calculated_exposure,
    })?;

    let sector = parse_sector(&po.symbol);
    let (base_currency, quote_currency) = parse_currencies(&po.symbol);
    let is_buy = po.side == 1;
    let base_size = if is_buy { calculated_exposure } else { -calculated_exposure };
    let quote_size = if is_buy { -calculated_exposure } else { calculated_exposure };

    exposure.dispatch(ExposureEvent::PositionOpened {
        position_id: position_uuid,
        symbol_id: po.symbol.clone(),
        sector,
        base_currency,
        quote_currency,
        base_size,
        quote_size,
        margin_used,
        risk_amount: margin_used * Decimal::new(2, 1),
    })?;

    sqlx::query(
        r#"
        INSERT INTO positions (position_id, symbol, side, state, initial_volume, current_volume, entry_price, opened_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        ON CONFLICT (position_id) DO NOTHING
        "#
    )
    .bind(po.position_id.clone())
    .bind(&po.symbol)
    .bind(if is_buy { "buy" } else { "sell" })
    .bind("open")
    .bind(volume)
    .bind(volume)
    .bind(entry_price)
    .execute(pool)
    .await?;

    Ok(())
}

async fn handle_position_closed(
    pc: &apex_protos::events::PositionClosedEvent,
    portfolio: &PortfolioRegistry,
    exposure: &ExposureRegistry,
    pool: &PgPool,
) -> Result<()> {
    let position_uuid = Uuid::parse_str(&pc.position_id).context("Invalid position UUID")?;
    
    let row_opt = sqlx::query("SELECT symbol, side, current_volume, entry_price FROM positions WHERE position_id = $1")
        .bind(&pc.position_id)
        .fetch_optional(pool)
        .await?;

    let (symbol, side, volume, entry_price) = if let Some(r) = row_opt {
        let sym: String = r.get("symbol");
        let sd: String = r.get("side");
        let vol: Decimal = r.get("current_volume");
        let ep: Decimal = r.get("entry_price");
        (sym, sd, vol, ep)
    } else {
        return Err(anyhow::anyhow!("Position not found in DB for release calculations"));
    };

    let contract_size = if symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("XAU") {
        Decimal::ONE
    } else {
        Decimal::from(100_000)
    };

    let exposure_released = volume * entry_price * contract_size;
    let margin_released = exposure_released / Decimal::from(100);
    let net_pnl = pc.net_pnl.as_ref()
        .map(|m| m.amount.parse::<Decimal>().unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);

    portfolio.dispatch(PortfolioEvent::PositionClosed {
        position_id: position_uuid,
        realized_pnl: net_pnl,
        margin_released,
        exposure_released,
    })?;

    let sector = parse_sector(&symbol);
    let (base_currency, quote_currency) = parse_currencies(&symbol);
    let is_buy = side == "buy";
    let base_size_released = if is_buy { exposure_released } else { -exposure_released };
    let quote_size_released = if is_buy { -exposure_released } else { exposure_released };

    exposure.dispatch(ExposureEvent::PositionClosed {
        position_id: position_uuid,
        symbol_id: symbol,
        sector,
        base_currency,
        quote_currency,
        base_size_released,
        quote_size_released,
        margin_released,
        risk_released: margin_released * Decimal::new(2, 1),
    })?;

    sqlx::query(
        r#"
        UPDATE positions 
        SET state = 'closed', realized_pnl = $1, current_volume = 0, updated_at = NOW()
        WHERE position_id = $2
        "#
    )
    .bind(net_pnl)
    .bind(&pc.position_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn handle_tick_received(
    tick: &apex_protos::events::TickReceivedEvent,
    portfolio: &PortfolioRegistry,
    exposure: &ExposureRegistry,
    pool: &PgPool,
) -> Result<()> {
    let bid_str = tick.bid.as_ref().map(|d| d.value.clone()).unwrap_or_else(|| "0".to_string());
    let ask_str = tick.ask.as_ref().map(|d| d.value.clone()).unwrap_or_else(|| "0".to_string());
    let bid = bid_str.parse::<Decimal>().unwrap_or(Decimal::ZERO);
    let ask = ask_str.parse::<Decimal>().unwrap_or(Decimal::ZERO);
    
    if bid.is_zero() && ask.is_zero() {
        return Ok(());
    }

    let symbol = tick.symbol.clone();
    if symbol.is_empty() {
        return Ok(());
    }

    let rows = sqlx::query("SELECT position_id, side, current_volume, entry_price, unrealized_pnl FROM positions WHERE symbol = $1 AND state = 'open'")
        .bind(&symbol)
        .fetch_all(pool)
        .await?;

    let contract_size = if symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("XAU") {
        Decimal::ONE
    } else {
        Decimal::from(100_000)
    };

    for r in rows {
        let position_id_str: String = r.get("position_id");
        let position_uuid = Uuid::parse_str(&position_id_str).unwrap_or_default();
        let side: String = r.get("side");
        let volume: Decimal = r.get("current_volume");
        let entry_price: Decimal = r.get("entry_price");
        
        let old_pnl_val: Option<Decimal> = r.get("unrealized_pnl");
        let old_pnl = old_pnl_val.unwrap_or(Decimal::ZERO);

        let is_buy = side == "buy";
        let current_price = if is_buy { bid } else { ask };
        let new_pnl = if is_buy {
            (current_price - entry_price) * volume * contract_size
        } else {
            (entry_price - current_price) * volume * contract_size
        };

        let pnl_delta = new_pnl - old_pnl;

        if !pnl_delta.is_zero() {
            let _ = portfolio.dispatch(PortfolioEvent::PnlUpdate {
                position_id: position_uuid,
                pnl_delta,
            });

            let _ = exposure.dispatch(ExposureEvent::PnlChanged {
                position_id: position_uuid,
                symbol_id: symbol.clone(),
                pnl_delta,
            });

            sqlx::query("UPDATE positions SET current_price = $1, unrealized_pnl = $2, updated_at = NOW() WHERE position_id = $3")
                .bind(current_price)
                .bind(new_pnl)
                .bind(&position_id_str)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}

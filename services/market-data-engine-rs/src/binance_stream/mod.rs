use crate::tick::Tick;
use crate::streaming::TickStream;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use rust_decimal::Decimal;
use std::str::FromStr;

pub struct BinanceSpotStream {
    symbol: String,
    receiver: Option<mpsc::UnboundedReceiver<Tick>>,
    _task: Option<tokio::task::JoinHandle<()>>,
}

impl BinanceSpotStream {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol: symbol.to_lowercase(),
            receiver: None,
            _task: None,
        }
    }
}

#[async_trait]
impl TickStream for BinanceSpotStream {
    async fn connect(&mut self) -> Result<(), String> {
        let url = format!("wss://stream.binance.com:9443/ws/{}@trade", self.symbol);
        
        let (ws_stream, _) = connect_async(&url).await.map_err(|e| e.to_string())?;
        let (_, mut read) = ws_stream.split();
        
        let (tx, rx) = mpsc::unbounded_channel();
        self.receiver = Some(rx);
        
        let symbol_upper = self.symbol.to_uppercase();
        
        let task = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let (Some(p), Some(v), Some(t)) = (
                            json["p"].as_str(),
                            json["q"].as_str(),
                            json["E"].as_i64(),
                        ) {
                            if let (Ok(price), Ok(volume)) = (Decimal::from_str(p), Decimal::from_str(v)) {
                                let tick = Tick {
                                    symbol: symbol_upper.clone(),
                                    timestamp: t as u64,
                                    bid: price, // simplified for trade stream
                                    ask: price,
                                    last: price,
                                    volume,
                                };
                                let _ = tx.send(tick);
                            }
                        }
                    }
                }
            }
        });
        
        self._task = Some(task);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        if let Some(task) = self._task.take() {
            task.abort();
        }
        self.receiver = None;
        Ok(())
    }

    async fn next_tick(&mut self) -> Option<Tick> {
        if let Some(rx) = &mut self.receiver {
            rx.recv().await
        } else {
            None
        }
    }
}

pub struct BinanceFuturesStream {
    symbol: String,
    receiver: Option<mpsc::UnboundedReceiver<Tick>>,
    _task: Option<tokio::task::JoinHandle<()>>,
}

impl BinanceFuturesStream {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol: symbol.to_lowercase(),
            receiver: None,
            _task: None,
        }
    }
}

#[async_trait]
impl TickStream for BinanceFuturesStream {
    async fn connect(&mut self) -> Result<(), String> {
        let url = format!("wss://fstream.binance.com/ws/{}@trade", self.symbol);
        
        let (ws_stream, _) = connect_async(&url).await.map_err(|e| e.to_string())?;
        let (_, mut read) = ws_stream.split();
        
        let (tx, rx) = mpsc::unbounded_channel();
        self.receiver = Some(rx);
        
        let symbol_upper = self.symbol.to_uppercase();
        
        let task = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let (Some(p), Some(v), Some(t)) = (
                            json["p"].as_str(),
                            json["q"].as_str(),
                            json["E"].as_i64(),
                        ) {
                            if let (Ok(price), Ok(volume)) = (Decimal::from_str(p), Decimal::from_str(v)) {
                                let tick = Tick {
                                    symbol: symbol_upper.clone(),
                                    timestamp: t as u64,
                                    bid: price,
                                    ask: price,
                                    last: price,
                                    volume,
                                };
                                let _ = tx.send(tick);
                            }
                        }
                    }
                }
            }
        });
        
        self._task = Some(task);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        if let Some(task) = self._task.take() {
            task.abort();
        }
        self.receiver = None;
        Ok(())
    }

    async fn next_tick(&mut self) -> Option<Tick> {
        if let Some(rx) = &mut self.receiver {
            rx.recv().await
        } else {
            None
        }
    }
}

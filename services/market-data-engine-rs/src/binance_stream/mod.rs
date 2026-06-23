use crate::tick::Tick;
use crate::streaming::TickStream;
use async_trait::async_trait;
use std::collections::VecDeque;

pub struct BinanceSpotStream {
    _symbol: String,
    connected: bool,
    mock_queue: VecDeque<Tick>,
}

impl BinanceSpotStream {
    pub fn new(symbol: String) -> Self {
        Self {
            _symbol: symbol,
            connected: false,
            mock_queue: VecDeque::new(),
        }
    }

    pub fn inject_mock_tick(&mut self, tick: Tick) {
        self.mock_queue.push_back(tick);
    }
}

#[async_trait]
impl TickStream for BinanceSpotStream {
    async fn connect(&mut self) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        Ok(())
    }

    async fn next_tick(&mut self) -> Option<Tick> {
        if !self.connected {
            return None;
        }
        self.mock_queue.pop_front()
    }
}

pub struct BinanceFuturesStream {
    _symbol: String,
    connected: bool,
    mock_queue: VecDeque<Tick>,
}

impl BinanceFuturesStream {
    pub fn new(symbol: String) -> Self {
        Self {
            _symbol: symbol,
            connected: false,
            mock_queue: VecDeque::new(),
        }
    }

    pub fn inject_mock_tick(&mut self, tick: Tick) {
        self.mock_queue.push_back(tick);
    }
}

#[async_trait]
impl TickStream for BinanceFuturesStream {
    async fn connect(&mut self) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        Ok(())
    }

    async fn next_tick(&mut self) -> Option<Tick> {
        if !self.connected {
            return None;
        }
        self.mock_queue.pop_front()
    }
}

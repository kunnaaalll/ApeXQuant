use super::{MarketDataConnector, ConnectorError, BoxFuture};
use crate::state_machine::ConnectionState;
use crate::health::FeedHealthGrade;
use crate::latency::LatencyGrade;
use crate::quality::FeedQuality;

pub enum BinanceMarketType {
    Spot,
    Futures,
}

pub struct BinanceFeedAdapter {
    pub market_type: BinanceMarketType,
    pub connection_state: ConnectionState,
}

impl BinanceFeedAdapter {
    pub fn new(market_type: BinanceMarketType) -> Self {
        Self {
            market_type,
            connection_state: ConnectionState::Disconnected,
        }
    }
}

impl MarketDataConnector for BinanceFeedAdapter {
    fn connect(&mut self) -> BoxFuture<'_, Result<(), ConnectorError>> {
        Box::pin(async move {
            self.connection_state = self.connection_state.transition(ConnectionState::Connecting)
                .map_err(|_| ConnectorError::InvalidState)?;
            self.connection_state = self.connection_state.transition(ConnectionState::Synchronizing)
                .map_err(|_| ConnectorError::InvalidState)?;
            self.connection_state = self.connection_state.transition(ConnectionState::Healthy)
                .map_err(|_| ConnectorError::InvalidState)?;
            Ok(())
        })
    }

    fn disconnect(&mut self) -> BoxFuture<'_, Result<(), ConnectorError>> {
        Box::pin(async move {
            self.connection_state = self.connection_state.transition(ConnectionState::Disconnected)
                .map_err(|_| ConnectorError::InvalidState)?;
            Ok(())
        })
    }

    fn health(&self) -> FeedHealthGrade {
        FeedHealthGrade::Excellent
    }

    fn latency(&self) -> LatencyGrade {
        LatencyGrade::Excellent
    }

    fn symbol_status(&self, _symbol: &str) -> Result<ConnectionState, ConnectorError> {
        Ok(self.connection_state)
    }

    fn feed_quality(&self) -> FeedQuality {
        FeedQuality::Elite
    }

    fn connection_state(&self) -> ConnectionState {
        self.connection_state
    }
}

use crate::brokers::events::BrokerEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinanceEvent {
    Standard(BrokerEvent),
    BinanceSpecificEvent(String),
}

use serde::{Deserialize, Serialize};
use crate::brokers::events::BrokerEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinanceEvent {
    Standard(BrokerEvent),
    BinanceSpecificEvent(String),
}

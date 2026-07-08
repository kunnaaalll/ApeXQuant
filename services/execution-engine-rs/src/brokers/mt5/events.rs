use crate::brokers::events::BrokerEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mt5Event {
    Standard(BrokerEvent),
    Mt5SpecificEvent(String),
}

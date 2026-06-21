use serde::{Deserialize, Serialize};
use crate::brokers::events::BrokerEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mt5Event {
    Standard(BrokerEvent),
    Mt5SpecificEvent(String),
}

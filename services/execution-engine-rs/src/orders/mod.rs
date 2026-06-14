pub mod cancel_orders;
pub mod limit_orders;
pub mod market_orders;
pub mod modify_orders;
pub mod stop_orders;
pub mod models;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}


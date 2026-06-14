use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use time::OffsetDateTime;
use crate::{RiskAssessment, TradeResult};

use std::future::Future;
use std::pin::Pin;
// Storage Module
pub mod postgres;

pub use postgres::PostgresShadowStorage;
pub trait ShadowStorage: Send + Sync {
    fn store_comparison<'a>(
        &'a self,
        trades: &'a [TradeResult],
        assessment: &'a RiskAssessment,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::error::RiskError>> + Send + 'a>>;
}

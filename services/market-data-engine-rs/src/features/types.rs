use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureWindow {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    Daily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMetadata {
    pub name: String,
    pub description: String,
    pub window: FeatureWindow,
    pub version: FeatureVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub atr: Option<Decimal>,
    pub realized_volatility: Option<Decimal>,
    pub spread: Option<Decimal>,
    pub momentum: Option<Decimal>,
    pub ema_distance: Option<Decimal>,
    pub correlation: Option<Decimal>,
    pub regime: Option<Decimal>, 
    pub session: Option<Decimal>, 
    pub market_quality: Option<Decimal>,
    pub trend_strength: Option<Decimal>,
    pub breakout_state: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSnapshot {
    pub symbol: String,
    pub window: FeatureWindow,
    pub metadata: FeatureMetadata,
    pub features: FeatureVector,
    pub timestamp: DateTime<Utc>,
}

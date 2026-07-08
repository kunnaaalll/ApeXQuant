pub mod emitter;
pub mod generator;
pub mod result;
pub mod validator;

use crate::market_data::Candle;
use std::collections::HashMap;

pub use emitter::SignalEmitter;
pub use generator::SignalGenerator;
pub use result::SignalResult;
pub use validator::SignalValidator;

/// Complete market context for signal generation
#[derive(Debug, Clone)]
pub struct MarketContext {
    pub symbol: String,
    pub candles: HashMap<String, Vec<Candle>>,
}

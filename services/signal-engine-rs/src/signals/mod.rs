//! Signal generation and orchestration

pub mod emitter;
pub mod generator;
pub mod result;
pub mod validator;

pub use generator::SignalGenerator;
pub use result::{SignalDirection, SignalResult, SignalStatus, SignalUpdate};
pub use validator::SignalValidator;

use crate::market_data::Candle;
use crate::mtf::MTFAlignmentResult;
use crate::regime::MarketRegime;
use crate::structure::StructureAnalysis;
use std::collections::HashMap;

/// Complete market context for signal generation
#[derive(Debug, Clone)]
pub struct MarketContext {
    /// Symbol
    pub symbol: String,
    /// Candles by timeframe
    pub candles: HashMap<String, Vec<Candle>>,
    /// Current market regime
    pub regime: MarketRegime,
    /// Market structure analysis
    pub structure: StructureAnalysis,
    /// Multi-timeframe alignment
    pub mtf_alignment: MTFAlignmentResult,
}

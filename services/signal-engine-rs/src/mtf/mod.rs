//! Multi-timeframe analysis

pub mod aligner;
pub mod hierarchy;
pub mod types;

pub use aligner::MTFAligner;
pub use hierarchy::TimeframeHierarchy;
pub use types::{MTFAlignmentResult, TimeframeAlignment};

use crate::config::Config;
use crate::market_data::Candle;
use crate::structure::StructureAnalysis;
use std::collections::HashMap;

/// Multi-timeframe analyzer
#[derive(Debug)]
pub struct MTFAnalyzer {
    config: Config,
}

impl MTFAnalyzer {
    /// Create a new MTF analyzer
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Analyze multi-timeframe alignment
    pub fn analyze(
        &self,
        candles: &HashMap<String, Vec<Candle>>,
        structure: &StructureAnalysis,
    ) -> crate::Result<MTFAlignmentResult> {
        let hierarchy = TimeframeHierarchy::new(&self.config.timeframes);
        let aligner = MTFAligner::new(&hierarchy);

        aligner.analyze(candles, structure)
    }
}

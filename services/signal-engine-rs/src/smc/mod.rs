//! Smart Money Concepts (SMC) detection

pub mod bos;
pub mod choch;
pub mod displacement;
pub mod fvg;
pub mod imbalance;
pub mod liquidity;
pub mod mitigation;
pub mod order_blocks;
pub mod premium_discount;

pub use bos::{detect_bos, has_recent_bos, BOSDirection, BOS};
pub use choch::{detect_choch, has_recent_choch, CHoCH, CHoCHDirection};
pub use displacement::{
    analyze_displacement_bias, detect_displacements, has_recent_displacement, Displacement,
    DisplacementDirection,
};
pub use fvg::{analyze_fvgs, detect_fvgs, FVGDirection, FairValueGap};
pub use imbalance::{analyze_imbalance, Imbalance, ImbalanceDirection};
pub use liquidity::{
    analyze_liquidity, detect_sweeps, has_recent_sweep, LiquiditySweep, SweepDirection,
};
pub use mitigation::{analyze_mitigations, check_mitigation, Mitigation, ReactionDirection};
pub use order_blocks::{
    detect_order_blocks, find_fresh_obs, get_entry_zone, OBDirection, OBType, OrderBlock,
};
pub use premium_discount::{
    analyze_premium_discount, calculate_premium_discount, PremiumDiscount, PriceZone,
};

use crate::market_data::Candle;
use crate::structure::swings::SwingPoint;
use std::collections::HashMap;

/// SMC pattern types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SMCPatternType {
    /// Break of Structure
    BOS,
    /// Change of Character
    CHoCH,
    /// Order Block
    OrderBlock,
    /// Fair Value Gap
    FVG,
    /// Liquidity Sweep
    LiquiditySweep,
    /// Displacement
    Displacement,
}

/// Common metadata for SMC patterns
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    /// Timeframe detected
    pub timeframe: String,
    /// Confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Age in bars
    pub age_bars: u32,
    /// Whether pattern is fresh
    pub fresh: bool,
}

/// Complete SMC analysis for a symbol
#[derive(Debug, Clone)]
pub struct SMCAnalysis {
    /// Detected BOS patterns
    pub bos_patterns: Vec<BOS>,
    /// Detected CHoCH patterns
    pub choch_patterns: Vec<CHoCH>,
    /// Detected order blocks
    pub order_blocks: Vec<OrderBlock>,
    /// Detected FVGs
    pub fvgs: Vec<FairValueGap>,
    /// Detected liquidity sweeps
    pub sweeps: Vec<LiquiditySweep>,
    /// Detected displacements
    pub displacements: Vec<Displacement>,
    /// Imbalance analysis
    pub imbalance: imbalance::ImbalanceAnalysis,
    /// Liquidity analysis
    pub liquidity: liquidity::LiquidityAnalysis,
    /// Premium/Discount analysis
    pub premium_discount: premium_discount::PremiumDiscountAnalysis,
}

impl SMCAnalysis {
    /// Create empty analysis
    pub fn empty() -> Self {
        Self {
            bos_patterns: Vec::new(),
            choch_patterns: Vec::new(),
            order_blocks: Vec::new(),
            fvgs: Vec::new(),
            sweeps: Vec::new(),
            displacements: Vec::new(),
            imbalance: imbalance::ImbalanceAnalysis {
                current: ImbalanceDirection::Neutral,
                strength: 0.0,
                recent_imbalances: Vec::new(),
                dominant_bias: ImbalanceDirection::Neutral,
            },
            liquidity: liquidity::LiquidityAnalysis {
                recent_high_sweeps: 0,
                recent_low_sweeps: 0,
                strongest_sweep: None,
                bias: None,
            },
            premium_discount: premium_discount::PremiumDiscountAnalysis {
                zone: PriceZone::Equilibrium,
                score: 0.0,
                favors_longs: false,
                favors_shorts: false,
                seeking_equilibrium: false,
            },
        }
    }

    /// Check for bullish structure
    pub fn is_bullish_structure(&self) -> bool {
        let bullish_bos = self
            .bos_patterns
            .iter()
            .any(|b| matches!(b.direction, BOSDirection::Bullish));
        let bullish_choch = self
            .choch_patterns
            .iter()
            .any(|c| matches!(c.direction, CHoCHDirection::Bullish));
        bullish_bos || bullish_choch
    }

    /// Check for bearish structure
    pub fn is_bearish_structure(&self) -> bool {
        let bearish_bos = self
            .bos_patterns
            .iter()
            .any(|b| matches!(b.direction, BOSDirection::Bearish));
        let bearish_choch = self
            .choch_patterns
            .iter()
            .any(|c| matches!(c.direction, CHoCHDirection::Bearish));
        bearish_bos || bearish_choch
    }

    /// Get freshest bullish order block
    pub fn freshest_bullish_ob(&self) -> Option<&OrderBlock> {
        self.order_blocks
            .iter()
            .filter(|ob| matches!(ob.direction, OBDirection::Bullish) && !ob.mitigated)
            .min_by_key(|ob| ob.age_bars)
    }

    /// Get freshest bearish order block
    pub fn freshest_bearish_ob(&self) -> Option<&OrderBlock> {
        self.order_blocks
            .iter()
            .filter(|ob| matches!(ob.direction, OBDirection::Bearish) && !ob.mitigated)
            .min_by_key(|ob| ob.age_bars)
    }

    /// Get freshest FVGs
    pub fn fresh_fvgs(&self, max_age: u32) -> Vec<&FairValueGap> {
        self.fvgs
            .iter()
            .filter(|f| !f.filled && f.age_bars <= max_age)
            .collect()
    }
}

/// SMC Engine for detecting all patterns
#[derive(Debug)]
pub struct SMCEngine;

impl SMCEngine {
    /// Create new SMC engine
    pub fn new() -> Self {
        Self
    }

    /// Analyze all SMC patterns for a symbol
    pub fn analyze(
        &self,
        candles: &HashMap<String, Vec<Candle>>,
        swings: &HashMap<String, Vec<SwingPoint>>,
    ) -> HashMap<String, SMCAnalysis> {
        let mut results = HashMap::new();

        for (timeframe, tf_candles) in candles {
            let tf_swings = swings.get(timeframe).cloned().unwrap_or_default();

            let analysis = self.analyze_timeframe(tf_candles, &tf_swings, timeframe);
            results.insert(timeframe.clone(), analysis);
        }

        results
    }

    /// Analyze single timeframe
    fn analyze_timeframe(
        &self,
        candles: &[Candle],
        swings: &[SwingPoint],
        timeframe: &str,
    ) -> SMCAnalysis {
        let bos_patterns = detect_bos(candles, swings, timeframe);
        let choch_patterns = detect_choch(candles, swings, timeframe);
        let order_blocks = detect_order_blocks(candles, swings, timeframe);
        let fvgs = detect_fvgs(candles, timeframe);
        let sweeps = detect_sweeps(candles, swings, timeframe);
        let displacements = detect_displacements(candles, timeframe, 2, 5, 1.5);

        SMCAnalysis {
            bos_patterns,
            choch_patterns,
            order_blocks,
            fvgs,
            sweeps,
            displacements,
            imbalance: analyze_imbalance(candles),
            liquidity: analyze_liquidity(candles, swings),
            premium_discount: analyze_premium_discount(candles),
        }
    }
}

impl Default for SMCEngine {
    fn default() -> Self {
        Self::new()
    }
}

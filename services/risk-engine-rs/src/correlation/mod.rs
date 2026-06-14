//! Correlation engine for portfolio exposure analysis
use rust_decimal::prelude::FromPrimitive;

use crate::RiskInputs;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod currency_exposure;
mod duplicate_positions;
mod matrix;
mod pair_correlation;
mod sector_exposure;
mod synthetic_exposure;

pub use currency_exposure::{CurrencyExposure, CurrencyExposureAnalyzer};
pub use duplicate_positions::DuplicatePositionDetector;
pub use matrix::CorrelationMatrix;
pub use pair_correlation::PairCorrelationEngine;
pub use sector_exposure::{Sector, SectorExposureAnalyzer};
pub use synthetic_exposure::SyntheticExposureAnalyzer;

/// Complete exposure analysis result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExposureAnalysis {
    /// Total exposure across all positions
    pub total_exposure: Decimal,

    /// Currency exposure breakdown
    pub currency_breakdown: HashMap<String, Decimal>,

    /// Sector exposure breakdown
    pub sector_breakdown: HashMap<Sector, Decimal>,

    /// Correlation score (0-1, higher = more risk from correlation)
    pub correlation_score: Decimal,

    /// Total number of positions
    pub total_positions: usize,

    /// Maximum correlation coefficient found
    pub max_correlation: Decimal,

    /// Risk concentration flags
    pub warnings: Vec<String>,

    /// Detailed reasoning
    pub reasons: Vec<String>,

    /// Hidden leverage detected
    pub hidden_leverage: Decimal,

    /// Synthetic positions detected
    pub synthetic_positions: Vec<String>,
}

impl ExposureAnalysis {
    /// Create empty analysis
    pub fn empty() -> Self {
        Self {
            total_exposure: Decimal::ZERO,
            currency_breakdown: HashMap::new(),
            sector_breakdown: HashMap::new(),
            correlation_score: Decimal::ZERO,
            total_positions: 0,
            max_correlation: Decimal::ZERO,
            warnings: Vec::new(),
            reasons: Vec::new(),
            hidden_leverage: Decimal::ZERO,
            synthetic_positions: Vec::new(),
        }
    }

    /// Check if exposure is within acceptable limits
    pub fn is_acceptable(&self, limit: Decimal) -> bool {
        self.total_exposure <= limit && self.correlation_score <= Decimal::from_f64(0.7).unwrap_or(Decimal::ONE)
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Add a reason
    pub fn add_reason(&mut self, reason: impl Into<String>) {
        self.reasons.push(reason.into());
    }
}

impl Default for ExposureAnalysis {
    fn default() -> Self {
        Self::empty()
    }
}

/// Correlation engine
pub struct CorrelationEngine {
    /// Currency exposure analyzer
    currency_analyzer: CurrencyExposureAnalyzer,
    /// Sector exposure analyzer
    sector_analyzer: SectorExposureAnalyzer,
    /// Pair correlation engine
    pair_correlation: PairCorrelationEngine,
    /// Synthetic exposure analyzer
    synthetic_analyzer: SyntheticExposureAnalyzer,
    /// Duplicate position detector
    duplicate_detector: DuplicatePositionDetector,
}

impl CorrelationEngine {
    /// Create new correlation engine
    pub fn new() -> Self {
        Self {
            currency_analyzer: CurrencyExposureAnalyzer::new(),
            sector_analyzer: SectorExposureAnalyzer::new(),
            pair_correlation: PairCorrelationEngine::default(),
            synthetic_analyzer: SyntheticExposureAnalyzer::new(),
            duplicate_detector: DuplicatePositionDetector::new(),
        }
    }

    /// Calculate correlation score for a potential trade
    pub fn score(&self, symbol: &str, open_positions: &[(String, Decimal, i8)]) -> Decimal {
        if open_positions.is_empty() {
            return Decimal::ZERO;
        }

        let correlations: Vec<Decimal> = open_positions
            .iter()
            .map(|(pos_symbol, _size, _direction)| {
                self.pair_correlation.correlation(symbol, pos_symbol)
            })
            .collect();

        if correlations.is_empty() {
            return Decimal::ZERO;
        }

        // Use average correlation as score, weighted by position size
        let total_correlation: Decimal = correlations.iter().sum();
        total_correlation / Decimal::from(correlations.len() as u32)
    }

    /// Perform comprehensive exposure analysis
    pub async fn analyze(
        &self,
        inputs: &RiskInputs,
        open_positions: &[(String, Decimal, i8)],
    ) -> ExposureAnalysis {
        let mut analysis = ExposureAnalysis::empty();

        // Currency exposure analysis
        let currency_exposure = self
            .currency_analyzer
            .analyze(&inputs.symbol, open_positions, inputs.direction);
        analysis.currency_breakdown = currency_exposure.breakdown;

        // Sector exposure
        let sector_exposure = self.sector_analyzer.analyze(&inputs.symbol, open_positions);
        analysis.sector_breakdown = sector_exposure.breakdown;

        // Synthetic exposure detection
        let synthetic = self.synthetic_analyzer.detect(&inputs.symbol, open_positions);
        analysis.synthetic_positions = synthetic.positions;
        analysis.hidden_leverage = synthetic.leverage_factor;

        // Duplicate detection
        let duplicates = self
            .duplicate_detector
            .detect(&inputs.symbol, inputs.direction, open_positions);
        if duplicates.has_duplicates {
            analysis.add_warning(format!("Duplicate position detected: {}", inputs.symbol));
        }

        // Calculate total exposure
        analysis.total_exposure = self.calculate_total_exposure(open_positions, inputs);
        analysis.total_positions = open_positions.len();

        // Calculate correlation score
        analysis.correlation_score = self.score(&inputs.symbol, open_positions);

        // Find max correlation
        analysis.max_correlation = self.calculate_max_correlation(&inputs.symbol, open_positions);

        // Generate warnings and reasons
        self.generate_warnings(&mut analysis, open_positions);

        analysis
    }

    /// Check if adding a position would create excessive correlation
    pub fn would_exceed_correlation(
        &self,
        symbol: &str,
        positions: &[(String, Decimal, i8)],
        max_correlation: Decimal,
    ) -> bool {
        for (pos_symbol, _size, _direction) in positions {
            let correlation = self.pair_correlation.correlation(symbol, pos_symbol);
            if correlation > max_correlation {
                return true;
            }
        }
        false
    }

    fn calculate_total_exposure(
        &self,
        positions: &[(String, Decimal, i8)],
        inputs: &RiskInputs,
    ) -> Decimal {
        let existing: Decimal = positions.iter().map(|(_, size, _)| size.abs()).sum();
        existing + inputs.equity.abs()
    }

    fn calculate_max_correlation(
        &self,
        symbol: &str,
        positions: &[(String, Decimal, i8)],
    ) -> Decimal {
        positions
            .iter()
            .map(|(pos_symbol, _, _)| self.pair_correlation.correlation(symbol, pos_symbol))
            .max()
            .unwrap_or(Decimal::ZERO)
    }

    fn generate_warnings(
        &self,
        analysis: &mut ExposureAnalysis,
        _positions: &[(String, Decimal, i8)],
    ) {
        // Check for currency concentration
        let breakdown = analysis.currency_breakdown.clone();
        for (currency, exposure) in breakdown {
            if exposure > Decimal::from_f64(0.6).unwrap_or(Decimal::ONE) {
                analysis.add_warning(format!("High {} exposure: {:.1}%", currency, exposure * Decimal::from(100)));
            }
        }

        // Check for high correlation
        if analysis.correlation_score > Decimal::from_f64(0.7).unwrap_or(Decimal::ONE) {
            analysis.add_warning("High correlation with existing positions");
        }

        // Check for hidden leverage
        if analysis.hidden_leverage > Decimal::from_f64(1.5).unwrap_or(Decimal::ONE) {
            analysis.add_warning(format!("Leverage amplification: {:.2}x", analysis.hidden_leverage));
        }

        // Generate reasons
        if analysis.correlation_score < Decimal::from_f64(0.3).unwrap_or(Decimal::ONE) {
            analysis.add_reason("Low correlation - good diversification");
        }

        if analysis.synthetic_positions.is_empty() {
            analysis.add_reason("No synthetic exposure detected");
        } else {
            analysis.add_reason(format!("Synthetic exposure: {:?}", analysis.synthetic_positions));
        }
    }
}

impl Default for CorrelationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Correlation metrics for a single position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionCorrelation {
    /// Symbol
    pub symbol: String,
    /// Correlation coefficient (-1 to 1)
    pub correlation: Decimal,
    /// Contribution to total correlation risk
    pub risk_contribution: Decimal,
}

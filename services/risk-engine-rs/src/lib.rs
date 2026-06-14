//! APEX V3 Risk Engine
//!
//! Dynamic, explainable, deterministic risk management for institutional trading.
//!
//! Core responsibilities:
//! - Position sizing (fixed fractional, Kelly, adaptive)
//! - Exposure management
//! - Drawdown protection
//! - Daily limits
//! - Correlation analysis
//! - Session risk adjustment
//! - Confidence scaling
//! - Circuit breakers

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::panic_in_result_fn)]

pub mod api;
pub mod circuit_breakers;
pub mod confidence;
pub mod correlation;
pub mod daily_limits;
pub mod drawdown;
pub mod error;
pub mod explanations;
pub mod exposure;
pub mod guards;
pub mod position_sizing;
pub mod profiles;
pub mod sessions;
pub mod storage;
pub mod streaks;
pub mod volatility;
pub mod config;
pub mod metrics;
pub mod health;
pub mod validation;

pub use apex_protos::risk::risk_engine_client::RiskEngineClient;
pub use circuit_breakers::{CircuitBreaker, CircuitBreakerRegistry, CircuitBreakerState};
pub use confidence::{ConfidenceEngine, ConfidenceInputs, ConfidenceScore};
pub use correlation::{CorrelationEngine, CurrencyExposure, ExposureAnalysis};
pub use daily_limits::{DailyLimitState, DailyLimitsEngine};
pub use drawdown::{DrawdownEngine, DrawdownState, DrawdownSnapshot};
pub use error::RiskError;
pub use explanations::{Explainable, RiskExplanation};
pub use exposure::{ExposureEngine, ExposureMetrics, TotalExposure};
pub use guards::RiskGuard;
pub use position_sizing::{
    FixedFractionalSizing, KellySizing, PositionSizeResult, PositionSizingEngine,
    VolatilityAdjustedSizing,
};
pub use profiles::{RiskProfile, RiskProfileEngine, RiskProfileSelector};
pub use sessions::{MarketSession, SessionEngine, SessionMetrics};
pub use storage::ShadowStorage;
pub use streaks::{StreakAnalyzer, StreakState};
pub use volatility::{VolatilityEngine, VolatilityMetrics};
pub use config::RiskEngineConfig;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::OffsetDateTime;
use tracing::{debug, info, warn};
use rust_decimal::prelude::ToPrimitive;

/// Maximum latency target for risk assessment
pub const MAX_LATENCY_MS: u64 = 5;

/// P99 latency target
pub const P99_LATENCY_MS: u64 = 15;

/// The key output from the Risk Engine - comprehensive risk assessment for a potential trade
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskAssessment {
    /// Whether the trade is approved
    pub approved: bool,

    /// Current risk profile applied
    pub risk_profile: RiskProfile,

    /// Risk percentage of capital
    pub risk_percent: Decimal,

    /// Calculated lot size
    pub lot_size: Decimal,

    /// Capital at risk
    pub capital_at_risk: Decimal,

    /// Exposure metrics
    pub exposure: ExposureMetrics,

    /// Correlation score (0-1, higher = more risk from correlation)
    pub correlation_score: Decimal,

    /// Current drawdown state
    pub drawdown_state: DrawdownState,

    /// Warnings that don't block but inform
    pub warnings: Vec<String>,

    /// Detailed reasoning
    pub reasoning: RiskExplanation,

    /// Assessment timestamp
    pub timestamp: OffsetDateTime,

    /// Processing latency in microseconds
    pub latency_us: u64,
}

impl RiskAssessment {
    /// Create a new denied assessment with explanations
    pub fn denied(reason: &str) -> Self {
        Self {
            approved: false,
            risk_profile: RiskProfile::Conservative,
            risk_percent: Decimal::ZERO,
            lot_size: Decimal::ZERO,
            capital_at_risk: Decimal::ZERO,
            exposure: ExposureMetrics::default(),
            correlation_score: Decimal::ZERO,
            drawdown_state: DrawdownState::Normal,
            warnings: Vec::new(),
            reasoning: RiskExplanation::single("denial", reason),
            timestamp: OffsetDateTime::now_utc(),
            latency_us: 0,
        }
    }
}

/// Primary inputs to the Risk Engine for assessment
#[derive(Debug, Clone)]
pub struct RiskInputs {
    /// Account equity
    pub equity: Decimal,

    /// Account balance
    pub balance: Decimal,

    /// Trading symbol
    pub symbol: String,

    /// Signal direction (1 for long, -1 for short)
    pub direction: i8,

    /// Entry price estimate
    pub entry_price: Decimal,

    /// Stop loss price
    pub stop_loss: Decimal,

    /// Take profit price
    pub take_profit: Option<Decimal>,

    /// Signal confidence (0-1)
    pub signal_confidence: Decimal,

    /// Confluence score (0-10)
    pub confluence_score: Decimal,

    /// Current regime quality (0-1)
    pub regime_quality: Decimal,

    /// Pattern quality score (0-1)
    pub pattern_quality: Decimal,

    /// ATR value for volatility
    pub atr: Option<Decimal>,

    /// Current spread
    pub spread: Decimal,

    /// Open positions (symbol, size, direction)
    pub open_positions: Vec<(String, Decimal, i8)>,

    /// Daily P&L
    pub daily_pnl: Decimal,

    /// Daily trade count
    pub daily_trades: u32,

    /// Recent trade history (last 20)
    pub recent_trades: Vec<TradeResult>,

    /// Current market session
    pub session: MarketSession,
}

/// Result of a completed trade
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeResult {
    /// Profit/loss
    pub pnl: Decimal,
    /// Whether it was a win
    pub is_win: bool,
    /// Duration in minutes
    pub duration_min: u32,
    /// Timestamp
    pub timestamp: OffsetDateTime,
}

/// Core Risk Engine
pub struct RiskEngine {
    config: RiskEngineConfig,
    position_sizing: Arc<PositionSizingEngine>,
    exposure: Arc<ExposureEngine>,
    drawdown: Arc<DrawdownEngine>,
    daily_limits: Arc<DailyLimitsEngine>,
    streaks: Arc<StreakAnalyzer>,
    confidence: Arc<ConfidenceEngine>,
    correlation: Arc<CorrelationEngine>,
    volatility: Arc<VolatilityEngine>,
    sessions: Arc<SessionEngine>,
    circuit_breakers: Arc<CircuitBreakerRegistry>,
    profiles: Arc<RiskProfileEngine>,
    guards: Arc<RiskGuard>,
    explanations: Arc<crate::explanations::ExplanationEngine>,
    storage: Option<Arc<dyn ShadowStorage + Send + Sync>>,
}


impl RiskEngine {
    /// Create a new Risk Engine
    pub fn new(config: RiskEngineConfig, storage: Option<Arc<dyn ShadowStorage + Send + Sync>>) -> Self {
        let drawdown = Arc::new(DrawdownEngine::new(
            Decimal::from_str_exact("0.05").unwrap_or(Decimal::new(5, 2)),
            Decimal::from_str_exact("0.03").unwrap_or(Decimal::new(3, 2)),
        ));
        let daily_limits = Arc::new(DailyLimitsEngine::new(
            config.daily_loss_limit_percent,
            10,
        ));
        Self {
            position_sizing: Arc::new(PositionSizingEngine::new(config.kelly_fraction)),
            exposure: Arc::new(ExposureEngine::new(config.max_simultaneous_trades)),
            drawdown: drawdown.clone(),
            daily_limits: daily_limits.clone(),
            streaks: Arc::new(StreakAnalyzer::new()),
            confidence: Arc::new(ConfidenceEngine::new()),
            correlation: Arc::new(CorrelationEngine::new()),
            volatility: Arc::new(VolatilityEngine::new()),
            sessions: Arc::new(SessionEngine::new()),
            circuit_breakers: Arc::new(CircuitBreakerRegistry::new()),
            profiles: Arc::new(RiskProfileEngine::new()),
            guards: Arc::new(RiskGuard::new(drawdown.clone(), daily_limits.clone())),
            explanations: Arc::new(crate::explanations::ExplanationEngine::new()),
            config,
            storage,
        }
    }

    /// Perform a comprehensive risk assessment
    pub async fn assess(&self, inputs: &RiskInputs) -> Result<RiskAssessment, RiskError> {
        let start = std::time::Instant::now();
        debug!(symbol = %inputs.symbol, "Starting risk assessment");

        // 1. Check circuit breakers first - can deny immediately
        if let Some(breaker) = self.circuit_breakers.check(inputs) {
            warn!(reason = %breaker.name(), "Circuit breaker triggered");
            crate::metrics::prometheus::record_circuit_breaker_trip(breaker.name());
            let mut assessment = RiskAssessment::denied(&format!("Circuit breaker: {}", breaker.name()));
            assessment.latency_us = start.elapsed().as_micros() as u64;
            return Ok(assessment);
        }

        // 2. Calculate overall confidence score
        let confidence = self.confidence.calculate(&ConfidenceInputs {
            signal_confidence: inputs.signal_confidence,
            confluence_score: inputs.confluence_score,
            regime_quality: inputs.regime_quality,
            pattern_quality: inputs.pattern_quality,
            session_quality: self.sessions.get_quality(&inputs.session),
        });

        // 3. Determine risk profile based on confidence and market conditions
        let profile = self.profiles.select(&confidence, &inputs.session);

        // 4. Evaluate drawdown state
        let drawdown_state = self.drawdown.evaluate(inputs.equity, inputs.balance);

        // 5. Check daily limits
        let daily_state = self.daily_limits.check(inputs.daily_pnl, inputs.daily_trades);

        // 6. Analyze streaks
        let streak_state = self.streaks.analyze(&inputs.recent_trades);

        // 7. Calculate exposure
        let exposure = self.exposure.calculate(inputs, &self.correlation).await?;

        // 8. Calculate correlation impact
        let correlation_score = self.correlation.score(&inputs.symbol, &inputs.open_positions);

        // 9. Get volatility adjustment
        let vol_metrics = self.volatility.analyze(inputs.atr, inputs.spread);

        // 10. Calculate position size
        let position_result = self.position_sizing.calculate(
            inputs,
            &profile,
            &drawdown_state,
            &daily_state,
            &streak_state,
            &exposure,
            &correlation_score,
            &vol_metrics,
        )?;

        // 11. Final guard check - validates all constraints
        if let Err(e) = self.guards.validate(
            &position_result,
            &exposure,
            &drawdown_state,
            &daily_state,
        ) {
            crate::metrics::prometheus::record_limit_exceeded(&e.to_string());
            let mut assessment = RiskAssessment::denied(&e.to_string());
            assessment.latency_us = start.elapsed().as_micros() as u64;
            return Ok(assessment);
        }

        // 12. Build comprehensive explanation
        let reasoning = self.explanations.build_comprehensive(
            &position_result,
            &drawdown_state,
            &exposure,
            &correlation_score,
            &streak_state,
            &vol_metrics,
        );

        // 13. Generate warnings
        let warnings = self.generate_warnings(&exposure, &drawdown_state, &correlation_score);

        let approved = position_result.lot_size > Decimal::ZERO;

        let latency_duration = start.elapsed();
        crate::metrics::prometheus::record_assessment_latency(latency_duration);
        crate::metrics::prometheus::update_active_exposure(&inputs.symbol, exposure.total_exposure.to_f64().unwrap_or(0.0));

        let assessment = RiskAssessment {
            approved,
            risk_profile: profile,
            risk_percent: position_result.risk_percent,
            lot_size: position_result.lot_size,
            capital_at_risk: position_result.capital_at_risk,
            exposure,
            correlation_score,
            drawdown_state,
            warnings,
            reasoning,
            timestamp: OffsetDateTime::now_utc(),
            latency_us: latency_duration.as_micros() as u64,
        };

        // Store shadow mode data
        if self.config.shadow_mode {
            if let Some(ref storage) = self.storage {
                let _ = storage.store_comparison(&inputs.recent_trades, &assessment).await;
            }
        }

        info!(
            symbol = %inputs.symbol,
            approved,
            lot_size = %assessment.lot_size,
            risk_percent = %assessment.risk_percent,
            latency_us = assessment.latency_us,
            "Risk assessment complete"
        );

        Ok(assessment)
    }

    fn generate_warnings(
        &self,
        exposure: &ExposureMetrics,
        drawdown: &DrawdownState,
        correlation: &Decimal,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        if exposure.total_positions >= (self.config.max_simultaneous_trades - 1) as usize {
            warnings.push("Near maximum position limit".to_string());
        }

        if matches!(drawdown, DrawdownState::Warning { .. }) {
            warnings.push("Drawdown approaching limits".to_string());
        }

        if *correlation > Decimal::from_str_exact("0.7").unwrap_or(Decimal::new(7, 1)) {
            warnings.push("High correlation with existing positions".to_string());
        }

        warnings
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_inputs() -> RiskInputs {
        RiskInputs {
            equity: Decimal::new(10000, 0),
            balance: Decimal::new(10000, 0),
            symbol: "EURUSD".to_string(),
            direction: 1,
            entry_price: Decimal::new(107500, 5),
            stop_loss: Decimal::new(107300, 5),
            take_profit: Some(Decimal::new(108000, 5)),
            signal_confidence: Decimal::from_str_exact("0.8").unwrap(),
            confluence_score: Decimal::from_str_exact("7.5").unwrap(),
            regime_quality: Decimal::from_str_exact("0.7").unwrap(),
            pattern_quality: Decimal::from_str_exact("0.75").unwrap(),
            atr: Some(Decimal::new(50, 5)),
            spread: Decimal::new(1, 5),
            open_positions: Vec::new(),
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: Vec::new(),
            session: MarketSession::London,
        }
    }

    #[test]
    fn test_risk_assessment_creation() {
        let assessment = RiskAssessment::denied("Test denial");
        assert!(!assessment.approved);
        assert_eq!(assessment.reasoning.explanations[0].value, "Test denial");
    }

    #[test]
    fn test_risk_engine_creation() {
        let config = RiskEngineConfig::default();
        let engine = RiskEngine::new(config, None);
        assert!(engine.config.shadow_mode);
    }
}

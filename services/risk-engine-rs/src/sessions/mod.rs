//! Market session analysis and risk adjustment
use rust_decimal::prelude::FromPrimitive;

use crate::RiskInputs;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, Time};

mod asia;
mod london;
mod new_york;
mod overlap;
mod session_classifier;

pub use asia::AsiaSession;
pub use london::LondonSession;
pub use new_york::NewYorkSession;
pub use overlap::SessionOverlap;
pub use session_classifier::SessionClassifier;

/// Market trading sessions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MarketSession {
    /// Asian session (Tokyo, Sydney)
    Asia,
    /// European session (London, Frankfurt)
    London,
    /// North American session (New York, Chicago)
    NewYork,
    /// London-New York overlap (highest liquidity)
    OverlapLondonNy,
    /// Sydney-Tokyo overlap
    OverlapAsia,
    /// Weekend/Sunday gap (low liquidity)
    Weekend,
    /// Low liquidity period
    LowLiquidity,
}

impl MarketSession {
    /// Get session quality score (0-1)
    pub fn quality_score(&self) -> Decimal {
        match self {
            MarketSession::OverlapLondonNy => Decimal::from_f64(1.0).unwrap(),
            MarketSession::London => Decimal::from_f64(0.9).unwrap(),
            MarketSession::NewYork => Decimal::from_f64(0.85).unwrap(),
            MarketSession::OverlapAsia => Decimal::from_f64(0.7).unwrap(),
            MarketSession::Asia => Decimal::from_f64(0.6).unwrap(),
            MarketSession::LowLiquidity => Decimal::from_f64(0.3).unwrap(),
            MarketSession::Weekend => Decimal::from_f64(0.1).unwrap(),
        }
    }

    /// Get volatility characteristics
    pub fn volatility_characteristic(&self) -> VolatilityCharacteristic {
        match self {
            MarketSession::OverlapLondonNy => VolatilityCharacteristic::High,
            MarketSession::London => VolatilityCharacteristic::Moderate,
            MarketSession::NewYork => VolatilityCharacteristic::Moderate,
            MarketSession::OverlapAsia => VolatilityCharacteristic::Moderate,
            MarketSession::Asia => VolatilityCharacteristic::Low,
            MarketSession::LowLiquidity => VolatilityCharacteristic::Variable,
            MarketSession::Weekend => VolatilityCharacteristic::Low,
        }
    }

    /// Get spread expectation
    pub fn expected_spread_multiplier(&self) -> Decimal {
        match self {
            MarketSession::OverlapLondonNy => Decimal::ONE,
            MarketSession::London => Decimal::from_f64(1.1).unwrap(),
            MarketSession::NewYork => Decimal::from_f64(1.15).unwrap(),
            MarketSession::OverlapAsia => Decimal::from_f64(1.3).unwrap(),
            MarketSession::Asia => Decimal::from_f64(1.5).unwrap(),
            MarketSession::LowLiquidity => Decimal::from_f64(2.0).unwrap(),
            MarketSession::Weekend => Decimal::from_f64(3.0).unwrap(),
        }
    }

    /// Get risk adjustment multiplier
    pub fn risk_multiplier(&self) -> Decimal {
        match self {
            MarketSession::OverlapLondonNy => Decimal::from_f64(1.1).unwrap(), // Best conditions
            MarketSession::London => Decimal::ONE,
            MarketSession::NewYork => Decimal::ONE,
            MarketSession::OverlapAsia => Decimal::from_f64(0.9).unwrap(),
            MarketSession::Asia => Decimal::from_f64(0.85).unwrap(),
            MarketSession::LowLiquidity => Decimal::from_f64(0.6).unwrap(),
            MarketSession::Weekend => Decimal::from_f64(0.3).unwrap(), // Avoid
        }
    }

    /// Whether this session should be avoided for new trades
    pub fn should_avoid(&self) -> bool {
        matches!(self, MarketSession::Weekend | MarketSession::LowLiquidity)
    }

    /// Get recommended max positions for this session
    pub fn max_positions(&self) -> u8 {
        match self {
            MarketSession::OverlapLondonNy => 5,
            MarketSession::London => 5,
            MarketSession::NewYork => 5,
            MarketSession::OverlapAsia => 4,
            MarketSession::Asia => 3,
            MarketSession::LowLiquidity => 2,
            MarketSession::Weekend => 0,
        }
    }

    /// Get description of the session
    pub fn description(&self) -> &'static str {
        match self {
            MarketSession::Asia => "Asian session - Tokyo/Sydney",
            MarketSession::London => "European session - London",
            MarketSession::NewYork => "North American session - New York",
            MarketSession::OverlapLondonNy => "London-New York overlap - peak liquidity",
            MarketSession::OverlapAsia => "Sydney-Tokyo overlap",
            MarketSession::Weekend => "Weekend - avoid trading",
            MarketSession::LowLiquidity => "Low liquidity period",
        }
    }
}

/// Volatility characteristics by session
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VolatilityCharacteristic {
    /// Low volatility expected
    Low,
    /// Moderate volatility
    Moderate,
    /// High volatility
    High,
    /// Variable/unpredictable
    Variable,
}

/// Session risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Current session
    pub current: MarketSession,
    /// Session quality (0-1)
    pub quality: Decimal,
    /// Liquidity score (0-1)
    pub liquidity_score: Decimal,
    /// Spread expectation
    pub spread_expectation: Decimal,
    /// Risk adjustment recommendation
    pub risk_adjustment: Decimal,
    /// Timestamp
    pub timestamp: OffsetDateTime,
}

/// Session risk engine
pub struct SessionEngine {
    classifier: SessionClassifier,
    london: LondonSession,
    ny: NewYorkSession,
    asia: AsiaSession,
    overlap: SessionOverlap,
}

impl SessionEngine {
    /// Create new session engine
    pub fn new() -> Self {
        Self {
            classifier: SessionClassifier::new(),
            london: LondonSession::new(),
            ny: NewYorkSession::new(),
            asia: AsiaSession::new(),
            overlap: SessionOverlap::new(),
        }
    }

    /// Detect current session from timestamp
    pub fn detect(&self, timestamp: OffsetDateTime) -> MarketSession {
        self.classifier.classify(timestamp)
    }

    /// Get quality score for a session
    pub fn get_quality(&self, session: &MarketSession) -> Decimal {
        session.quality_score()
    }

    /// Analyze session conditions
    pub fn analyze(&self, timestamp: OffsetDateTime) -> SessionMetrics {
        let session = self.detect(timestamp);

        SessionMetrics {
            current: session,
            quality: session.quality_score(),
            liquidity_score: self.calculate_liquidity(&session),
            spread_expectation: session.expected_spread_multiplier(),
            risk_adjustment: session.risk_multiplier(),
            timestamp,
        }
    }

    /// Get session-specific risk adjustments
    pub fn get_adjustments(&self, inputs: &RiskInputs) -> SessionAdjustments {
        let session = &inputs.session;

        SessionAdjustments {
            risk_multiplier: session.risk_multiplier(),
            spread_multiplier: session.expected_spread_multiplier(),
            position_limit: session.max_positions(),
            recommended: !session.should_avoid(),
            reasoning: session.description().to_string(),
        }
    }

    fn calculate_liquidity(&self, session: &MarketSession) -> Decimal {
        match session {
            MarketSession::OverlapLondonNy => Decimal::from_f64(1.0).unwrap(),
            MarketSession::London => Decimal::from_f64(0.9).unwrap(),
            MarketSession::NewYork => Decimal::from_f64(0.85).unwrap(),
            MarketSession::OverlapAsia => Decimal::from_f64(0.6).unwrap(),
            MarketSession::Asia => Decimal::from_f64(0.5).unwrap(),
            MarketSession::LowLiquidity => Decimal::from_f64(0.2).unwrap(),
            MarketSession::Weekend => Decimal::from_f64(0.05).unwrap(),
        }
    }

    /// Check if timestamp is in session transition
    pub fn is_session_transition(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();

        // Transitions happen at hour boundaries when sessions open/close
        let transition_hours = [
            Time::from_hms(7, 0, 0).unwrap(),   // Frankfurt pre-open
            Time::from_hms(8, 0, 0).unwrap(),   // London open
            Time::from_hms(13, 0, 0).unwrap(),  // NY pre-open
            Time::from_hms(14, 0, 0).unwrap(),  // NY open (overlap)
            Time::from_hms(17, 0, 0).unwrap(),  // London close
            Time::from_hms(22, 0, 0).unwrap(),  // NY close
        ];

        transition_hours
            .iter()
            .any(|&t| (time.hour() == t.hour()) && time.minute() < 15)
    }
}

impl Default for SessionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Session-specific risk adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAdjustments {
    /// Risk multiplier
    pub risk_multiplier: Decimal,
    /// Spread multiplier
    pub spread_multiplier: Decimal,
    /// Position limit
    pub position_limit: u8,
    /// Whether trading is recommended
    pub recommended: bool,
    /// Reasoning
    pub reasoning: String,
}

/// Global session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Asian session start (UTC)
    pub asia_start: Time,
    /// London session start (UTC)
    pub london_start: Time,
    /// New York session start (UTC)
    pub ny_start: Time,
    /// Enable session-based adjustments
    pub enabled: bool,
    /// Avoid weekend trading
    pub avoid_weekend: bool,
    /// Avoid low liquidity periods
    pub avoid_low_liquidity: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            asia_start: Time::from_hms(22, 0, 0).unwrap(), // Sydney open
            london_start: Time::from_hms(8, 0, 0).unwrap(),
            ny_start: Time::from_hms(14, 0, 0).unwrap(),
            enabled: true,
            avoid_weekend: true,
            avoid_low_liquidity: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_quality_scores() {
        assert!(
            MarketSession::OverlapLondonNy.quality_score()
                > MarketSession::London.quality_score()
        );
        assert!(
            MarketSession::London.quality_score() > MarketSession::Asia.quality_score()
        );
    }

    #[test]
    fn test_session_risk_multipliers() {
        assert!(
            MarketSession::OverlapLondonNy.risk_multiplier()
                >= MarketSession::London.risk_multiplier()
        );
        assert!(
            MarketSession::Weekend.risk_multiplier() < MarketSession::Asia.risk_multiplier()
        );
    }

    #[test]
    fn test_session_avoidance() {
        assert!(MarketSession::Weekend.should_avoid());
        assert!(!MarketSession::London.should_avoid());
    }

    #[test]
    fn test_session_classifier() {
        let engine = SessionEngine::new();

        // Test London time
        let london_time = OffsetDateTime::from_unix_timestamp(1605091200).unwrap();
        let session = engine.detect(london_time);

        // Just verify it returns a valid session
        assert!(!session.should_avoid() || session == MarketSession::Weekend);
    }
}

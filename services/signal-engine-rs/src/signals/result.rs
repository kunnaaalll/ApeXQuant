//! Signal result types

use crate::config::SignalQuality;
use crate::mtf::types::MarketBias;
use crate::regime::RegimeType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Signal generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    /// Unique signal identifier
    pub signal_id: Uuid,
    /// Symbol/pair
    pub symbol: String,
    /// Trade direction
    pub direction: SignalDirection,
    /// Confidence score (0-100)
    pub confidence: f64,
    /// Confluence score (0-100)
    pub confluence_score: u8,
    /// Signal quality grade
    pub quality: SignalQuality,
    /// Current market regime
    pub market_regime: RegimeType,
    /// Multi-timeframe alignment bias
    pub timeframe_alignment: MarketBias,
    /// Entry zone (top)
    #[serde(with = "rust_decimal::serde::str")]
    pub entry_zone_top: Decimal,
    /// Entry zone (bottom)
    #[serde(with = "rust_decimal::serde::str")]
    pub entry_zone_bottom: Decimal,
    /// Stop loss zone
    #[serde(with = "rust_decimal::serde::str")]
    pub stop_zone: Decimal,
    /// Take profit zone
    #[serde(with = "rust_decimal::serde::str")]
    pub target_zone: Decimal,
    /// Risk/reward ratio
    pub risk_reward: f64,
    /// Patterns that contributed to signal
    pub patterns_detected: Vec<DetectedPattern>,
    /// Evidence collection
    pub evidence: Vec<SignalEvidence>,
    /// Human-readable reasons
    pub reasons: Vec<String>,
    /// Signal generation timestamp
    pub timestamp: OffsetDateTime,
}

/// Signal direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalDirection {
    /// Long/Buy signal
    Long,
    /// Short/Sell signal
    Short,
}

/// Detected pattern info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern type name
    pub pattern_type: String,
    /// Pattern timeframe
    pub timeframe: String,
    /// Pattern strength
    pub strength: f64,
    /// Pattern location (price)
    #[serde(with = "rust_decimal::serde::str_option")]
    pub location: Option<Decimal>,
    /// Pattern confidence
    pub confidence: f64,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Signal evidence entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvidence {
    /// Evidence type
    pub evidence_type: String,
    /// Evidence description
    pub description: String,
    /// Supporting data
    pub data: serde_json::Value,
}

impl SignalResult {
    /// Create new signal with generated ID
    pub fn new(symbol: String, direction: SignalDirection) -> Self {
        Self {
            signal_id: Uuid::new_v4(),
            symbol,
            direction,
            confidence: 0.0,
            confluence_score: 0,
            quality: SignalQuality::Reject,
            market_regime: RegimeType::Undefined,
            timeframe_alignment: MarketBias::Neutral,
            entry_zone_top: Decimal::ZERO,
            entry_zone_bottom: Decimal::ZERO,
            stop_zone: Decimal::ZERO,
            target_zone: Decimal::ZERO,
            risk_reward: 0.0,
            patterns_detected: Vec::new(),
            evidence: Vec::new(),
            reasons: Vec::new(),
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    /// Check if signal is valid (meets minimum standards)
    pub fn is_valid(&self) -> bool {
        self.confluence_score > 0
            && self.quality != SignalQuality::Reject
            && self.confidence > 0.0
    }

    /// Calculate mid entry price
    pub fn entry_mid(&self) -> Decimal {
        (self.entry_zone_top + self.entry_zone_bottom) / Decimal::from(2)
    }

    /// Get position size suggestion (placeholder)
    pub fn position_size_risk_percent(&self, account_balance: Decimal, risk_percent: f64) -> Decimal {
        let risk_amount = account_balance * Decimal::from_f64_retain(risk_percent / 100.0).unwrap_or_default();
        let stop_pips = (self.entry_mid() - self.stop_zone).abs();

        if stop_pips == Decimal::ZERO {
            return Decimal::ZERO;
        }

        // Simplified - real implementation would consider pip value
        risk_amount / stop_pips
    }

    /// Convert to summary string
    pub fn summary(&self) -> String {
        format!(
            "{} {} {:.1}% (Q:{:?}, C:{}%, RR:{:.1}:1)",
            self.symbol,
            match self.direction {
                SignalDirection::Long => "LONG",
                SignalDirection::Short => "SHORT",
            },
            self.confidence,
            self.quality,
            self.confluence_score,
            self.risk_reward
        )
    }
}

/// Signal update for shadow mode comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalUpdate {
    /// Signal ID
    pub signal_id: Uuid,
    /// Updated confidence
    pub confidence: f64,
    /// Price when updated
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// Update timestamp
    pub timestamp: OffsetDateTime,
    /// Status
    pub status: SignalStatus,
}

/// Signal lifecycle status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalStatus {
    /// Signal generated
    Generated,
    /// Signal confirmed
    Confirmed,
    /// Signal invalidated
    Invalidated,
    /// Target hit
    TargetHit,
    /// Stop hit
    StopHit,
    /// Closed manually
    Closed,
    /// Expired
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = SignalResult::new("EURUSD".to_string(), SignalDirection::Long);

        assert_eq!(signal.symbol, "EURUSD");
        assert!(matches!(signal.direction, SignalDirection::Long));
        assert!(!signal.is_valid()); // Not valid until scored
    }

    #[test]
    fn test_signal_summary() {
        let mut signal = SignalResult::new("EURUSD".to_string(), SignalDirection::Long);
        signal.confidence = 75.0;
        signal.quality = SignalQuality::A;
        signal.confluence_score = 75;
        signal.risk_reward = 2.5;

        let summary = signal.summary();
        assert!(summary.contains("EURUSD"));
        assert!(summary.contains("LONG"));
        assert!(summary.contains("75.0%"));
    }
}

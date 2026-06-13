//! Signal validation and quality checks

use crate::config::{SignalQuality, Config};
use crate::signals::SignalResult;

/// Validates signals against quality criteria
#[derive(Debug)]
pub struct SignalValidator {
    config: Config,
}

impl SignalValidator {
    /// Create new validator
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Validate signal against all criteria
    pub fn validate(&self, signal: &SignalResult) -> ValidationResult {
        let mut checks = Vec::new();
        let mut passed = true;

        // Check confluence score
        let confluence_pass = signal.confluence_score >= self.config.min_confluence_score;
        checks.push(ValidationCheck {
            name: "confluence".to_string(),
            passed: confluence_pass,
            message: format!("Confluence: {} (min: {})", signal.confluence_score, self.config.min_confluence_score),
        });
        passed &= confluence_pass;

        // Check signal quality
        let quality_pass = signal.quality.meets(self.config.min_signal_quality);
        checks.push(ValidationCheck {
            name: "quality".to_string(),
            passed: quality_pass,
            message: format!("Quality: {:?} (min: {:?})", signal.quality, self.config.min_signal_quality),
        });
        passed &= quality_pass;

        // Check risk/reward
        let rr_pass = signal.risk_reward >= self.config.min_risk_reward;
        checks.push(ValidationCheck {
            name: "risk_reward".to_string(),
            passed: rr_pass,
            message: format!("R:R: {:.2} (min: {:.1})", signal.risk_reward, self.config.min_risk_reward),
        });
        passed &= rr_pass;

        // Check confidence
        let confidence_pass = signal.confidence >= 50.0;
        checks.push(ValidationCheck {
            name: "confidence".to_string(),
            passed: confidence_pass,
            message: format!("Confidence: {:.1}%", signal.confidence),
        });
        passed &= confidence_pass;

        ValidationResult {
            valid: passed,
            checks,
        }
    }

    /// Quick check if signal passes minimum standards
    pub fn quick_check(&self, signal: &SignalResult) -> bool {
        signal.confluence_score >= self.config.min_confluence_score
            && signal.quality.meets(self.config.min_signal_quality)
            && signal.risk_reward >= self.config.min_risk_reward
            && signal.confidence >= 50.0
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall valid status
    pub valid: bool,
    /// Individual check results
    pub checks: Vec<ValidationCheck>,
}

/// Individual validation check
#[derive(Debug, Clone)]
pub struct ValidationCheck {
    /// Check name
    pub name: String,
    /// Whether check passed
    pub passed: bool,
    /// Check message
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::result::SignalDirection;
    use rust_decimal::Decimal;
    use time::OffsetDateTime;
    use uuid::Uuid;

    fn create_test_signal() -> SignalResult {
        SignalResult {
            signal_id: Uuid::new_v4(),
            symbol: "EURUSD".to_string(),
            direction: SignalDirection::Long,
            confidence: 75.0,
            confluence_score: 75,
            quality: SignalQuality::A,
            market_regime: crate::regime::RegimeType::TrendingUp,
            timeframe_alignment: crate::mtf::types::MarketBias::Bullish,
            entry_zone_top: Decimal::new(10500, 2),
            entry_zone_bottom: Decimal::new(10450, 2),
            stop_zone: Decimal::new(10300, 2),
            target_zone: Decimal::new(10800, 2),
            risk_reward: 2.5,
            patterns_detected: vec![],
            evidence: vec![],
            reasons: vec!["test".to_string()],
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn test_validation_pass() {
        let config = Config::default();
        let validator = SignalValidator::new(&config);
        let signal = create_test_signal();

        let result = validator.validate(&signal);
        assert!(result.valid);
    }

    #[test]
    fn test_validation_fail_low_score() {
        let config = Config::default();
        let validator = SignalValidator::new(&config);
        let mut signal = create_test_signal();
        signal.confluence_score = 50;

        let result = validator.validate(&signal);
        assert!(!result.valid);
    }
}

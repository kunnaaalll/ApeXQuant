use crate::intelligence::MarketIntelligenceProfile;
use crate::regime::MarketRegime;
use crate::correlation::CorrelationMetrics;

pub struct ValidationFramework;

impl ValidationFramework {
    pub fn validate_determinism(profile1: &MarketIntelligenceProfile, profile2: &MarketIntelligenceProfile) -> bool {
        profile1 == profile2
    }

    pub fn validate_replay(live_profile: &MarketIntelligenceProfile, replay_profile: &MarketIntelligenceProfile) -> Result<(), &'static str> {
        if live_profile != replay_profile {
            return Err("Replay determinism failed: Profiles do not match");
        }
        Ok(())
    }

    pub fn validate_bounds(profile: &MarketIntelligenceProfile) -> Result<(), &'static str> {
        if profile.market_score > 100 { return Err("Market score out of bounds"); }
        if profile.confidence_score > 100 { return Err("Confidence score out of bounds"); }
        if profile.tradeability_score > 100 { return Err("Tradeability score out of bounds"); }
        if profile.volatility.score > 100 { return Err("Volatility score out of bounds"); }
        if profile.trend.strength_score > 100 { return Err("Trend strength score out of bounds"); }
        if profile.correlation.score > 100 { return Err("Correlation score out of bounds"); }
        Ok(())
    }

    pub fn validate_regime(regime: MarketRegime) -> bool {
        matches!(regime, 
            MarketRegime::TrendFollowing | 
            MarketRegime::MeanReversion | 
            MarketRegime::Breakout | 
            MarketRegime::Expansion | 
            MarketRegime::Compression | 
            MarketRegime::RiskOn | 
            MarketRegime::RiskOff | 
            MarketRegime::Transitional
        )
    }

    pub fn validate_correlation(metrics: &CorrelationMetrics) -> Result<(), &'static str> {
        if metrics.score > 100 {
            return Err("Correlation score out of bounds");
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod phase5_tests;

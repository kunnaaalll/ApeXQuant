#[cfg(test)]
mod tests {
    use crate::quality::quality_score::{PortfolioQuality, PortfolioQualityState, QualityEvent, PortfolioQualityBreakdown, QualityContribution};

    fn default_breakdown() -> PortfolioQualityBreakdown {
        let contrib = QualityContribution { weight: 0.0, score: 0.0, reason: "".to_string() };
        PortfolioQualityBreakdown {
            win_rate: contrib.clone(),
            profit_factor: contrib.clone(),
            expectancy: contrib.clone(),
            average_rr: contrib.clone(),
            position_quality: contrib.clone(),
            position_health: contrib.clone(),
            capital_efficiency: contrib.clone(),
            trade_efficiency: contrib.clone(),
            holding_efficiency: contrib.clone(),
            allocation_efficiency: contrib.clone(),
            recovery_factor: contrib.clone(),
            recent_performance: contrib.clone(),
            drawdown_efficiency: contrib,
        }
    }

    #[test]
    fn test_quality_initialization() {
        let quality = PortfolioQuality::new(1000);
        assert_eq!(quality.current_score, 50.0);
        assert_eq!(quality.state, PortfolioQualityState::Neutral);
        assert_eq!(quality.version, 1);
    }

    #[test]
    fn test_quality_state_boundaries() {
        assert_eq!(PortfolioQuality::determine_state(95.0), PortfolioQualityState::Excellent);
        assert_eq!(PortfolioQuality::determine_state(90.0), PortfolioQualityState::Excellent);
        assert_eq!(PortfolioQuality::determine_state(89.9), PortfolioQualityState::Good);
        assert_eq!(PortfolioQuality::determine_state(75.0), PortfolioQualityState::Good);
        assert_eq!(PortfolioQuality::determine_state(74.9), PortfolioQualityState::Neutral);
        assert_eq!(PortfolioQuality::determine_state(50.0), PortfolioQualityState::Neutral);
        assert_eq!(PortfolioQuality::determine_state(49.9), PortfolioQualityState::Weak);
        assert_eq!(PortfolioQuality::determine_state(25.0), PortfolioQualityState::Weak);
        assert_eq!(PortfolioQuality::determine_state(24.9), PortfolioQualityState::Critical);
        assert_eq!(PortfolioQuality::determine_state(0.0), PortfolioQualityState::Critical);
    }

    #[test]
    fn test_apply_event_clamping() {
        let mut quality = PortfolioQuality::new(1000);
        let breakdown = default_breakdown();
        
        let snapshot_high = quality.apply_event(QualityEvent::PnLChanged, 150.0, breakdown.clone(), 1001);
        assert_eq!(snapshot_high.composite_score, 100.0);
        assert_eq!(snapshot_high.state, PortfolioQualityState::Excellent);

        let snapshot_low = quality.apply_event(QualityEvent::PnLChanged, -50.0, breakdown, 1002);
        assert_eq!(snapshot_low.composite_score, 0.0);
        assert_eq!(snapshot_low.state, PortfolioQualityState::Critical);
    }

    #[test]
    fn test_apply_decay() {
        let mut quality = PortfolioQuality::new(1000);
        let breakdown = default_breakdown();
        quality.apply_event(QualityEvent::PnLChanged, 80.0, breakdown, 1001); // State is Good
        
        // Apply 10 points decay
        let snapshot = quality.apply_decay(10.0, 1002);
        assert_eq!(snapshot.composite_score, 70.0);
        assert_eq!(snapshot.state, PortfolioQualityState::Neutral); // Dropped to Neutral
        assert_eq!(snapshot.version, 3);
    }
}

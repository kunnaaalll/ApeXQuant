use rust_decimal::Decimal;
use crate::health::health_score::{PortfolioHealth, PortfolioHealthState, HealthEvent, PortfolioHealthBreakdown, HealthContribution};

    fn default_breakdown() -> PortfolioHealthBreakdown {
        let contrib = HealthContribution { weight: Decimal::ZERO, contribution: Decimal::ZERO, reason: "".to_string() };
        PortfolioHealthBreakdown {
            portfolio_heat: contrib.clone(),
            drawdown: contrib.clone(),
            margin_utilization: contrib.clone(),
            leverage: contrib.clone(),
            open_risk: contrib.clone(),
            exposure_concentration: contrib.clone(),
            correlation_pressure: contrib.clone(),
            recovery_state: contrib.clone(),
            circuit_breakers: contrib.clone(),
            capital_reserves: contrib.clone(),
            volatility_regime: contrib.clone(),
            position_quality: contrib.clone(),
            portfolio_quality: contrib,
        }
    }

    #[test]
    fn test_health_initialization() {
        let health = PortfolioHealth::new(1000);
        assert_eq!(health.current_score, 50);
        assert_eq!(health.state, PortfolioHealthState::Normal);
        assert_eq!(health.version, 1);
    }

    #[test]
    fn test_health_state_boundaries() {
        assert_eq!(PortfolioHealth::determine_state(100), PortfolioHealthState::Excellent);
        assert_eq!(PortfolioHealth::determine_state(90), PortfolioHealthState::Excellent);
        assert_eq!(PortfolioHealth::determine_state(89), PortfolioHealthState::Healthy);
        assert_eq!(PortfolioHealth::determine_state(75), PortfolioHealthState::Healthy);
        assert_eq!(PortfolioHealth::determine_state(74), PortfolioHealthState::Normal);
        assert_eq!(PortfolioHealth::determine_state(50), PortfolioHealthState::Normal);
        assert_eq!(PortfolioHealth::determine_state(49), PortfolioHealthState::Weak);
        assert_eq!(PortfolioHealth::determine_state(25), PortfolioHealthState::Weak);
        assert_eq!(PortfolioHealth::determine_state(24), PortfolioHealthState::Critical);
        assert_eq!(PortfolioHealth::determine_state(0), PortfolioHealthState::Critical);
    }

    #[test]
    fn test_apply_event_clamping() {
        let mut health = PortfolioHealth::new(1000);
        let breakdown = default_breakdown();
        
        // Use a value > 100
        let snapshot_high = health.apply_event(HealthEvent::PnLChanged, 150, breakdown.clone(), 1001);
        assert_eq!(snapshot_high.composite_score, 100);
        assert_eq!(snapshot_high.state, PortfolioHealthState::Excellent);

        // Can't test < 0 directly because u8 is unsigned, so compiler enforces 0-255. 
        // 0 is the min.
        let snapshot_low = health.apply_event(HealthEvent::PnLChanged, 0, breakdown, 1002);
        assert_eq!(snapshot_low.composite_score, 0);
        assert_eq!(snapshot_low.state, PortfolioHealthState::Critical);
    }

    #[test]
    fn test_apply_recovery() {
        let mut health = PortfolioHealth::new(1000);
        let breakdown = default_breakdown();
        health.apply_event(HealthEvent::PnLChanged, 10, breakdown, 1001); // State is Critical
        
        // Apply 20 points recovery, but safe_recovery limits it to 5 per tick.
        let snapshot = health.apply_recovery(20, 1002);
        assert_eq!(snapshot.composite_score, 15);
        assert_eq!(snapshot.state, PortfolioHealthState::Critical); // Still Critical
        assert_eq!(snapshot.version, 3);
        
        // Apply another recovery of 5
        let snapshot2 = health.apply_recovery(5, 1003);
        assert_eq!(snapshot2.composite_score, 20);
        assert_eq!(snapshot2.state, PortfolioHealthState::Critical);
    }

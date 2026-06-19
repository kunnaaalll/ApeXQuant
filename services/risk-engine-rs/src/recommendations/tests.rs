use crate::recommendations::committee::evaluate_committee;
use crate::recommendations::consistency::{validate_consistency, ConsistencyError};
use crate::recommendations::models::{
    CircuitBreakerState, CorrelationSeverity, DrawdownState, ExposureState, HiddenLeverage,
    RiskCommitteeDecision, RiskInputs, RiskRecommendation, TailRiskScore, TradeAdmissionPolicy,
    VarSeverity,
};
use crate::recommendations::snapshot::RecommendationSnapshot;

    fn get_healthy_inputs() -> RiskInputs {
        RiskInputs {
            drawdown_state: DrawdownState::Healthy,
            exposure_state: ExposureState::Healthy,
            correlation_severity: CorrelationSeverity::Healthy,
            var_severity: VarSeverity::Healthy,
            circuit_breaker_state: CircuitBreakerState::Healthy,
            tail_risk_score: TailRiskScore::Healthy,
            hidden_leverage: HiddenLeverage::Healthy,
            exposure_concentration: ExposureState::Healthy,
        }
    }

    #[test]
    fn test_frozen_always_dominates() {
        let mut inputs = get_healthy_inputs();
        inputs.drawdown_state = DrawdownState::Frozen;
        // Even if other inputs are healthy, frozen should dominate.
        let decision = evaluate_committee(&inputs, 100);
        assert_eq!(decision.recommendation, RiskRecommendation::FreezeTrading);
        assert_eq!(decision.admission_policy, TradeAdmissionPolicy::Freeze);

        inputs.drawdown_state = DrawdownState::Healthy;
        inputs.circuit_breaker_state = CircuitBreakerState::Frozen;
        let decision2 = evaluate_committee(&inputs, 101);
        assert_eq!(decision2.recommendation, RiskRecommendation::FreezeTrading);
        assert_eq!(decision2.admission_policy, TradeAdmissionPolicy::Freeze);
    }

    #[test]
    fn test_emergency_reduction_priority() {
        let mut inputs = get_healthy_inputs();
        inputs.exposure_state = ExposureState::Collapse;
        let decision = evaluate_committee(&inputs, 100);
        assert_eq!(
            decision.recommendation,
            RiskRecommendation::EmergencyReduction
        );
        // It must be at least blocked
        assert!(
            decision.admission_policy == TradeAdmissionPolicy::Block
                || decision.admission_policy == TradeAdmissionPolicy::Freeze
        );
    }

    #[test]
    fn test_inconsistent_states_rejected() {
        // Mock an impossible decision manually and test the validator
        let decision1 = RiskCommitteeDecision {
            recommendation: RiskRecommendation::IncreaseRisk,
            admission_policy: TradeAdmissionPolicy::Freeze,
            explanation: evaluate_committee(&get_healthy_inputs(), 0)
                .explanation,
            confidence: 100,
            timestamp: 0,
        };
        assert_eq!(
            validate_consistency(&decision1),
            Err(ConsistencyError::IncreaseWithFreeze)
        );

        let decision2 = RiskCommitteeDecision {
            recommendation: RiskRecommendation::EmergencyReduction,
            admission_policy: TradeAdmissionPolicy::Allow,
            explanation: decision1.explanation.clone(),
            confidence: 100,
            timestamp: 0,
        };
        assert_eq!(
            validate_consistency(&decision2),
            Err(ConsistencyError::AllowWithEmergencyReduction)
        );
    }

    #[test]
    fn test_trade_blocking() {
        let mut inputs = get_healthy_inputs();
        inputs.var_severity = VarSeverity::Critical;
        let decision = evaluate_committee(&inputs, 100);
        assert_eq!(decision.admission_policy, TradeAdmissionPolicy::Block);
    }

    #[test]
    fn test_recommendation_determinism_100k() {
        let inputs = get_healthy_inputs();
        let expected_decision = evaluate_committee(&inputs, 100);

        for _ in 0..100_000 {
            let decision = evaluate_committee(&inputs, 100);
            assert_eq!(decision, expected_decision);
        }
    }

    #[test]
    fn test_snapshot_rebuild() {
        let inputs = get_healthy_inputs();
        let decision = evaluate_committee(&inputs, 123456);

        let snapshot = RecommendationSnapshot::rebuild(
            decision.recommendation,
            decision.admission_policy,
            decision.explanation.clone(),
            decision.timestamp,
            1,
        );

        assert_eq!(snapshot.recommendation, decision.recommendation);
        assert_eq!(snapshot.admission_policy, decision.admission_policy);
        assert_eq!(snapshot.explanation, decision.explanation);
        assert_eq!(snapshot.timestamp, decision.timestamp);
        assert_eq!(snapshot.version, 1);
    }

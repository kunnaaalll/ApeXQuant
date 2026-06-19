use crate::explanations::summary::generate_narrative;
use crate::recommendations::models::{
    CircuitBreakerState, CorrelationSeverity, DrawdownState, ExposureState, HiddenLeverage,
    RiskInputs, TailRiskScore, TradeAdmissionPolicy, VarSeverity,
};

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
fn test_explanation_has_no_empty_fields() {
    let inputs = get_healthy_inputs();
    let narrative =
        generate_narrative(&inputs, TradeAdmissionPolicy::Allow, "Because".to_string());

    let explanation = narrative.explanation;
    assert!(!explanation.why.is_empty());
    assert!(!explanation.what_improved.is_empty());
    assert!(!explanation.dominant_factor.is_empty());
    assert!(!explanation.prevented_stronger_recommendation.is_empty());
    assert!(!explanation.what_deteriorated.is_empty());
}

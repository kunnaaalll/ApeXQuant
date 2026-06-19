use super::confidence_levels::ConfidenceLevel;
use super::expected_shortfall::ExpectedShortfallAssessment;
use super::historical_var::HistoricalVaR;
use super::parametric_var::ParametricVaR;
use super::tail_risk::TailRiskAssessment;
use rust_decimal::Decimal;

#[test]
fn test_historical_var_calculations() {
    let mut hvar = HistoricalVaR::new(100);
    // Add returns: 0.05, 0.01, -0.02, -0.05, -0.10
    hvar.add_return(Decimal::new(5, 2));
    hvar.add_return(Decimal::new(1, 2));
    hvar.add_return(Decimal::new(-2, 2));
    hvar.add_return(Decimal::new(-5, 2));
    hvar.add_return(Decimal::new(-10, 2));

    let var_90 = hvar.compute_var(ConfidenceLevel::Ninety);
    // 5 elements. 90% confidence = 10th percentile. 5 * 0.10 = 0.5 -> index 0.
    // Sorted returns: -0.10, -0.05, -0.02, 0.01, 0.05
    assert_eq!(var_90, Decimal::new(10, 2));

    assert_eq!(hvar.worst_case_loss(), Decimal::new(10, 2));
}

#[test]
fn test_parametric_var_calculations() {
    let mut pvar = ParametricVaR::new();
    pvar.add_return(Decimal::new(5, 2));
    pvar.add_return(Decimal::new(1, 2));
    pvar.add_return(Decimal::new(-2, 2));
    pvar.add_return(Decimal::new(-5, 2));
    pvar.add_return(Decimal::new(-10, 2));

    let var_95 = pvar.compute_var(ConfidenceLevel::NinetyFive);
    assert!(var_95 > Decimal::ZERO);
}

#[test]
fn test_expected_shortfall_invariant() {
    let mut es = ExpectedShortfallAssessment::new(Decimal::new(5, 2));
    es.add_loss(Decimal::new(6, 2));
    es.add_loss(Decimal::new(10, 2));

    let shortfall = es.compute_shortfall();
    // Avg of 0.06 and 0.10 is 0.08
    assert_eq!(shortfall, Decimal::new(8, 2));
    assert!(shortfall >= Decimal::new(5, 2));
}

#[test]
fn test_tail_risk_score_clamping() {
    let mut tr = TailRiskAssessment::new();
    tr.record_tail_loss(Decimal::new(15, 1));
    tr.record_tail_loss(Decimal::new(25, 1));
    
    let score = tr.tail_risk_score(2);
    // Expected to be clamped at 100
    assert!(score <= 100);
    assert!(tr.get_largest_loss() >= tr.average_tail_loss());
}

#[test]
fn test_deterministic_iterations() {
    let mut pvar = ParametricVaR::new();
    let mut tr = TailRiskAssessment::new();

    let start_val = Decimal::new(1, 2);
    for i in 0..100_000 {
        let ret = if i % 2 == 0 { start_val } else { -start_val };
        pvar.add_return(ret);
        if ret < Decimal::ZERO {
            tr.record_tail_loss(-ret);
        }
    }

    assert!(pvar.variance() > Decimal::ZERO);
    assert_eq!(tr.get_largest_loss(), start_val);
}

#[test]
fn test_zero_panics_and_no_nan() {
    // Tests that division by zero or negative counts don't panic
    let pvar = ParametricVaR::new();
    assert_eq!(pvar.variance(), Decimal::ZERO);
    assert_eq!(pvar.standard_deviation(), Decimal::ZERO);
    assert_eq!(pvar.compute_var(ConfidenceLevel::Ninety), Decimal::ZERO);
    
    let tr = TailRiskAssessment::new();
    assert_eq!(tr.average_tail_loss(), Decimal::ZERO);
    assert_eq!(tr.frequency_of_extreme_events(0), Decimal::ZERO);
}

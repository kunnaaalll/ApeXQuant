use rust_decimal_macros::dec;

use super::drawdown_state::DrawdownState;
use super::metrics::UlcerIndexAssessment;
use super::decay::DrawdownRecoveryModel;

#[test]
fn test_state_transitions() {
    let normal = DrawdownState::Normal;
    let warning = DrawdownState::Warning;
    let frozen = DrawdownState::Frozen;
    let recovery = DrawdownState::Recovery;

    assert!(normal.can_transition_to(&warning));
    assert!(normal.can_transition_to(&frozen));
    
    // Frozen can only go to recovery
    assert!(frozen.can_transition_to(&recovery));
    assert!(!frozen.can_transition_to(&normal));
    assert!(!frozen.can_transition_to(&warning));
}

#[test]
fn test_ulcer_index_calculation() {
    let drawdowns = vec![dec!(0.0), dec!(5.0), dec!(10.0), dec!(5.0), dec!(0.0)];
    let ulcer = UlcerIndexAssessment::calculate(&drawdowns);
    
    // UI = sqrt( (0 + 25 + 100 + 25 + 0) / 5 ) = sqrt(150 / 5) = sqrt(30) = 5.477
    assert!((ulcer.ulcer_index - dec!(5.477)).abs() < dec!(0.01));
    assert_eq!(ulcer.depth, dec!(10.0));
    assert_eq!(ulcer.duration, 5);
    assert_eq!(ulcer.persistence, dec!(0.6)); // 3 out of 5 periods > 0
}

#[test]
fn test_drawdown_recovery_model() {
    let mut model = DrawdownRecoveryModel::new(10, dec!(0.9));
    
    assert_eq!(model.consecutive_positive_periods, 0);
    assert!(!model.is_stable());
    
    model.update(true, dec!(1.0));
    assert_eq!(model.consecutive_positive_periods, 1);
    
    // Model should gradually recover
    for _ in 0..50 {
        model.update(true, dec!(1.0));
    }
    
    assert!(model.is_stable());
    
    // Instant penalization
    model.update(false, dec!(-1.0));
    assert_eq!(model.consecutive_positive_periods, 0);
    assert!(!model.is_stable());
}

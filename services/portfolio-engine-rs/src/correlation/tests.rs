use super::matrix::{CorrelationMatrix, CorrelationType, CorrelationWindow};
use super::leverage::{HiddenLeverageAssessment, SyntheticDuplicate, ThemeConcentration};

#[test]
fn test_correlation_matrix_initialization() {
    let ids = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
    let matrix = CorrelationMatrix::new(CorrelationType::Symbol, CorrelationWindow::ShortTerm, ids.clone());

    assert_eq!(matrix.rows, 3);
    assert_eq!(matrix.cols, 3);
    
    // Diagonal should be 1.0
    assert_eq!(matrix.get_correlation(0, 0), Some(1.0));
    assert_eq!(matrix.get_correlation(1, 1), Some(1.0));
    assert_eq!(matrix.get_correlation(2, 2), Some(1.0));
    
    // Off-diagonal should be 0.0 initially
    assert_eq!(matrix.get_correlation(0, 1), Some(0.0));
}

#[test]
fn test_correlation_matrix_updates() {
    let ids = vec!["BTC".to_string(), "ETH".to_string()];
    let mut matrix = CorrelationMatrix::new(CorrelationType::Symbol, CorrelationWindow::MediumTerm, ids);
    
    matrix.set_correlation(0, 1, 0.85);
    
    assert_eq!(matrix.get_correlation(0, 1), Some(0.85));
    // Check symmetry
    assert_eq!(matrix.get_correlation(1, 0), Some(0.85));
}

#[test]
fn test_hidden_leverage_assessment() {
    let mut assessment = HiddenLeverageAssessment::new();
    
    assert!(!assessment.has_hidden_leverage);
    
    assessment.synthetic_duplicates.push(SyntheticDuplicate {
        symbols: vec!["BTC".to_string(), "ETH".to_string()],
        correlation_score: 0.90,
        combined_exposure_pct: 15.0,
    });
    
    assessment.theme_concentration.push(ThemeConcentration {
        theme: "Risk-On".to_string(),
        symbols: vec!["SPY".to_string(), "BTC".to_string()],
        total_exposure_pct: 40.0,
    });
    
    assessment.assess();
    
    assert!(assessment.has_hidden_leverage);
    assert_eq!(assessment.total_hidden_leverage_ratio, 13.5); // 15.0 * 0.90
}

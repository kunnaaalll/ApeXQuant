use super::matrix::{CorrelationMatrix, CorrelationType, CorrelationWindow};
use super::leverage::{HiddenLeverageAssessment, SyntheticDuplicate, ThemeConcentration};
use rust_decimal::Decimal;

#[test]
fn test_correlation_matrix_initialization() {
    let ids = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
    let matrix = CorrelationMatrix::new(CorrelationType::Symbol, CorrelationWindow::ShortTerm, ids.clone());

    assert_eq!(matrix.rows, 3);
    assert_eq!(matrix.cols, 3);
    
    // Diagonal should be 1.0
    assert_eq!(matrix.get_correlation(0, 0), Some(Decimal::ONE));
    assert_eq!(matrix.get_correlation(1, 1), Some(Decimal::ONE));
    assert_eq!(matrix.get_correlation(2, 2), Some(Decimal::ONE));
    
    // Off-diagonal should be 0.0 initially
    assert_eq!(matrix.get_correlation(0, 1), Some(Decimal::ZERO));
}

#[test]
fn test_correlation_matrix_updates() {
    let ids = vec!["BTC".to_string(), "ETH".to_string()];
    let mut matrix = CorrelationMatrix::new(CorrelationType::Symbol, CorrelationWindow::MediumTerm, ids);
    
    matrix.set_correlation(0, 1, Decimal::new(85, 2));
    
    assert_eq!(matrix.get_correlation(0, 1), Some(Decimal::new(85, 2)));
    // Check symmetry
    assert_eq!(matrix.get_correlation(1, 0), Some(Decimal::new(85, 2)));
}

#[test]
fn test_hidden_leverage_assessment() {
    let mut assessment = HiddenLeverageAssessment::new();
    
    assert!(!assessment.has_hidden_leverage);
    
    assessment.synthetic_duplicates.push(SyntheticDuplicate {
        symbols: vec!["BTC".to_string(), "ETH".to_string()],
        correlation_score: Decimal::new(90, 2),
        combined_exposure_pct: Decimal::new(150, 1),
    });
    
    assessment.theme_concentration.push(ThemeConcentration {
        theme: "Risk-On".to_string(),
        symbols: vec!["SPY".to_string(), "BTC".to_string()],
        total_exposure_pct: Decimal::new(400, 1),
    });
    
    assessment.assess();
    
    assert!(assessment.has_hidden_leverage);
    assert_eq!(assessment.total_hidden_leverage_ratio, Decimal::new(135, 1)); // 15.0 * 0.90
}

use super::opportunity_ranking::OpportunityRanking;
use super::weakness_ranking::WeaknessLevel;
use rust_decimal::Decimal;

#[test]
fn test_rankings() {
    let edge = Decimal::from(2);
    let confidence = Decimal::from(3);
    let sample_quality = Decimal::from(4);

    let ranking = OpportunityRanking::calculate(edge, confidence, sample_quality);
    assert_eq!(ranking.score, Decimal::from(24));

    let weakness = WeaknessLevel::Danger;
    assert_eq!(weakness, WeaknessLevel::Danger);
}

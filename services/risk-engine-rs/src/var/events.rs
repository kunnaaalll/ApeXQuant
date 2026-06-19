use rust_decimal::Decimal;
use super::confidence_levels::ConfidenceLevel;
use super::Severity;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VarRiskEvent {
    HistoricalVarUpdated {
        level: ConfidenceLevel,
        var_value: Decimal,
    },
    ParametricVarUpdated {
        level: ConfidenceLevel,
        var_value: Decimal,
    },
    ExpectedShortfallUpdated {
        level: ConfidenceLevel,
        shortfall_value: Decimal,
    },
    TailRiskChanged {
        score: u32, // bounded 0 -> 100
    },
    ConfidenceLevelUpdated {
        level: ConfidenceLevel,
    },
    SeverityChanged {
        old_severity: Severity,
        new_severity: Severity,
    },
}

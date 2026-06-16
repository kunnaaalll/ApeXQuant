use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::CorrelationSeverity;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationCategory {
    CurrencyLongUSD,
    CurrencyShortUSD,
    RiskOnCrypto,
    RiskOnTech,
    RiskOnGrowth,
    RiskOffBonds,
    RiskOffMetals,
    RiskOffDefensive,
    Inflation,
    Commodity,
    Sector(String),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationCluster {
    pub category: CorrelationCategory,
    pub members: Vec<String>,
    pub weight: Decimal,
    pub concentration: Decimal,
    pub severity: CorrelationSeverity,
}

impl CorrelationCluster {
    pub fn new(
        category: CorrelationCategory,
        members: Vec<String>,
        weight: Decimal,
        concentration: Decimal,
    ) -> Self {
        let zero = Decimal::new(0, 0);
        let hundred = Decimal::new(100, 0);

        // Invariants: weight >= 0, concentration ∈ [0, 100]
        let bounded_weight = weight.max(zero);
        let bounded_concentration = concentration.clamp(zero, hundred);

        let mut cluster = Self {
            category,
            members,
            weight: bounded_weight,
            concentration: bounded_concentration,
            severity: CorrelationSeverity::Low,
        };

        cluster.evaluate_severity();
        cluster
    }

    fn evaluate_severity(&mut self) {
        // Evaluate severity based on concentration and weight combinations
        // Example thresholds:
        let crit_conc = Decimal::new(80, 0);
        let high_conc = Decimal::new(50, 0);
        let elev_conc = Decimal::new(30, 0);

        if self.concentration >= crit_conc && self.weight >= Decimal::new(5, 0) {
            self.severity = CorrelationSeverity::Critical;
        } else if self.concentration >= high_conc {
            self.severity = CorrelationSeverity::High;
        } else if self.concentration >= elev_conc {
            self.severity = CorrelationSeverity::Elevated;
        } else if self.concentration > Decimal::new(10, 0) {
            self.severity = CorrelationSeverity::Moderate;
        } else {
            self.severity = CorrelationSeverity::Low;
        }
    }
}

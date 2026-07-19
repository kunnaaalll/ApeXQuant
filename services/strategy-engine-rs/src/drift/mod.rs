use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftState {
    Improving,
    Stable,
    Weakening,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftEngine {
    pub edge_drift: Decimal,
    pub expectancy_drift: Decimal,
    pub confidence_drift: Decimal,
    pub stability_drift: Decimal,
}

impl DriftEngine {
    pub fn new() -> Self {
        Self {
            edge_drift: Decimal::from(0),
            expectancy_drift: Decimal::from(0),
            confidence_drift: Decimal::from(0),
            stability_drift: Decimal::from(0),
        }
    }
}

impl Default for DriftEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftEngine {
    pub fn state(&self) -> DriftState {
        let min_drift = self
            .edge_drift
            .min(self.expectancy_drift)
            .min(self.confidence_drift)
            .min(self.stability_drift);

        let max_drift = self
            .edge_drift
            .max(self.expectancy_drift)
            .max(self.confidence_drift)
            .max(self.stability_drift);

        let collapse_threshold = Decimal::new(-60, 2); // -0.60
        let critical_threshold = Decimal::new(-40, 2); // -0.40
        let weakening_threshold = Decimal::new(-20, 2); // -0.20
        let improving_threshold = Decimal::new(10, 2); // +0.10

        if min_drift <= collapse_threshold {
            DriftState::Collapse
        } else if min_drift <= critical_threshold {
            DriftState::Critical
        } else if min_drift <= weakening_threshold {
            DriftState::Weakening
        } else if max_drift >= improving_threshold {
            DriftState::Improving
        } else {
            DriftState::Stable
        }
    }
}

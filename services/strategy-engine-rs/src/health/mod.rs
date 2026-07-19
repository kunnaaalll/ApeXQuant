use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthState {
    Collapse,
    Danger,
    Weak,
    Normal,
    Strong,
    Healthy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthScore {
    score: Decimal,
}

impl HealthScore {
    pub fn new(score: Decimal) -> Self {
        let clamped = score.clamp(Decimal::from(0), Decimal::from(100));
        Self { score: clamped }
    }

    pub fn value(&self) -> Decimal {
        self.score
    }

    pub fn calculate(
        drawdown: Decimal,
        degradation: Decimal,
        confidence: Decimal,
        expectancy: Decimal,
        stability: Decimal,
    ) -> Self {
        let base = Decimal::from(100);
        let penalty = drawdown + degradation;
        let bonus = (confidence + expectancy + stability) / Decimal::from(3);

        Self::new(base - penalty + bonus)
    }

    pub fn state(&self) -> HealthState {
        if self.score >= Decimal::from(90) {
            HealthState::Healthy
        } else if self.score >= Decimal::from(75) {
            HealthState::Strong
        } else if self.score >= Decimal::from(50) {
            HealthState::Normal
        } else if self.score >= Decimal::from(30) {
            HealthState::Weak
        } else if self.score >= Decimal::from(10) {
            HealthState::Danger
        } else {
            HealthState::Collapse
        }
    }
}

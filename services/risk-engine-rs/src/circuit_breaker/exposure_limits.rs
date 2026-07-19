use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExposureState {
    Normal = 0,
    Elevated = 1,
    Danger = 2,
    Critical = 3,
    Frozen = 4,
}

#[derive(Debug, Clone)]
pub struct ExposureLimitAssessment {
    pub gross_exposure: Decimal,
    pub net_exposure: Decimal,
    pub max_symbol_concentration: Decimal,
    pub max_currency_concentration: Decimal,
    pub max_theme_concentration: Decimal,
    pub state: ExposureState,
}

impl ExposureLimitAssessment {
    pub fn new(
        gross_exposure: Decimal,
        net_exposure: Decimal,
        max_symbol_concentration: Decimal,
        max_currency_concentration: Decimal,
        max_theme_concentration: Decimal,
    ) -> Self {
        let mut assessment = Self {
            gross_exposure,
            net_exposure,
            max_symbol_concentration,
            max_currency_concentration,
            max_theme_concentration,
            state: ExposureState::Normal,
        };
        assessment.update_state();
        assessment
    }

    fn update_state(&mut self) {
        // Assume thresholds:
        // Concentration > 50% = Frozen
        // Concentration > 40% = Critical
        // Concentration > 30% = Danger
        // Concentration > 20% = Elevated
        // Else = Normal

        let max_conc = self
            .max_symbol_concentration
            .max(self.max_currency_concentration)
            .max(self.max_theme_concentration);

        self.state = if max_conc >= Decimal::new(50, 2) {
            ExposureState::Frozen
        } else if max_conc >= Decimal::new(40, 2) {
            ExposureState::Critical
        } else if max_conc >= Decimal::new(30, 2) {
            ExposureState::Danger
        } else if max_conc >= Decimal::new(20, 2) {
            ExposureState::Elevated
        } else {
            ExposureState::Normal
        };
    }
}

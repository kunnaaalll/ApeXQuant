use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HiddenLeverageState {
    Normal,
    Elevated,
    High,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenLeverageAssessment {
    pub synthetic_duplication: Decimal,
    pub directional_overlap: Decimal,
    pub currency_overlap: Decimal,
    pub theme_overlap: Decimal,
    pub sector_overlap: Decimal,
    pub total_hidden_leverage_ratio: Decimal,
    pub state: HiddenLeverageState,
}

impl HiddenLeverageAssessment {
    pub fn new(
        synthetic_duplication: Decimal,
        directional_overlap: Decimal,
        currency_overlap: Decimal,
        theme_overlap: Decimal,
        sector_overlap: Decimal,
    ) -> Self {
        let zero = Decimal::new(0, 0);

        // Enforce invariants
        let mut assessment = Self {
            synthetic_duplication: synthetic_duplication.max(zero),
            directional_overlap: directional_overlap.max(zero),
            currency_overlap: currency_overlap.max(zero),
            theme_overlap: theme_overlap.max(zero),
            sector_overlap: sector_overlap.max(zero),
            total_hidden_leverage_ratio: zero,
            state: HiddenLeverageState::Normal,
        };

        assessment.calculate_total_ratio();
        assessment.determine_state();

        assessment
    }

    fn calculate_total_ratio(&mut self) {
        // Base logic for total hidden leverage ratio.
        // It sums overlaps and duplicates with specific institutional weights.
        // E.g., synthetic duplication has a higher risk multiplier.
        let w_synth = Decimal::new(15, 1); // 1.5
        let w_dir = Decimal::new(12, 1);   // 1.2
        let w_curr = Decimal::new(10, 1);  // 1.0
        let w_theme = Decimal::new(8, 1);  // 0.8
        let w_sector = Decimal::new(5, 1); // 0.5

        self.total_hidden_leverage_ratio = (self.synthetic_duplication * w_synth)
            + (self.directional_overlap * w_dir)
            + (self.currency_overlap * w_curr)
            + (self.theme_overlap * w_theme)
            + (self.sector_overlap * w_sector);
    }

    fn determine_state(&mut self) {
        let ratio = self.total_hidden_leverage_ratio;

        let threshold_collapse = Decimal::new(400, 2); // >= 4.0
        let threshold_critical = Decimal::new(300, 2); // >= 3.0
        let threshold_high = Decimal::new(200, 2);     // >= 2.0
        let threshold_elevated = Decimal::new(120, 2); // >= 1.2

        if ratio >= threshold_collapse {
            self.state = HiddenLeverageState::Collapse;
        } else if ratio >= threshold_critical {
            self.state = HiddenLeverageState::Critical;
        } else if ratio >= threshold_high {
            self.state = HiddenLeverageState::High;
        } else if ratio >= threshold_elevated {
            self.state = HiddenLeverageState::Elevated;
        } else {
            self.state = HiddenLeverageState::Normal;
        }
    }
}

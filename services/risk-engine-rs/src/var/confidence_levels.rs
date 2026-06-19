use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConfidenceLevel {
    Ninety,
    NinetyFive,
    NinetyNine,
    NinetyNineNine,
}

impl ConfidenceLevel {
    /// Returns the z-score for the given confidence level (assuming a normal distribution).
    /// These are standard approximations.
    pub fn z_score(&self) -> Decimal {
        match self {
            ConfidenceLevel::Ninety => Decimal::new(128155, 5),
            ConfidenceLevel::NinetyFive => Decimal::new(164485, 5),
            ConfidenceLevel::NinetyNine => Decimal::new(232635, 5),
            ConfidenceLevel::NinetyNineNine => Decimal::new(309023, 5),
        }
    }

    /// Returns the percentile threshold for historical VaR calculations.
    pub fn percentile(&self) -> Decimal {
        match self {
            ConfidenceLevel::Ninety => Decimal::new(10, 2),
            ConfidenceLevel::NinetyFive => Decimal::new(5, 2),
            ConfidenceLevel::NinetyNine => Decimal::new(1, 2),
            ConfidenceLevel::NinetyNineNine => Decimal::new(1, 3),
        }
    }
}

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalScenario {
    FlashCrash,
    BlackMonday1987,
    CovidCrash2020,
    DotComBubble,
    Lehman2008,
    SyntheticExtreme,
}

impl HistoricalScenario {
    pub fn volatility_multiplier(&self) -> Decimal {
        match self {
            Self::FlashCrash => dec!(3.0),
            Self::BlackMonday1987 => dec!(5.0),
            Self::CovidCrash2020 => dec!(4.5),
            Self::DotComBubble => dec!(2.5),
            Self::Lehman2008 => dec!(4.0),
            Self::SyntheticExtreme => dec!(6.0),
        }
    }

    pub fn correlation_multiplier(&self) -> Decimal {
        match self {
            Self::FlashCrash => dec!(1.5),
            Self::BlackMonday1987 => dec!(2.0),
            Self::CovidCrash2020 => dec!(1.8),
            Self::DotComBubble => dec!(1.2),
            Self::Lehman2008 => dec!(1.9),
            Self::SyntheticExtreme => dec!(2.5),
        }
    }

    pub fn liquidity_reduction(&self) -> Decimal {
        match self {
            Self::FlashCrash => dec!(0.10),
            Self::BlackMonday1987 => dec!(0.05),
            Self::CovidCrash2020 => dec!(0.20),
            Self::DotComBubble => dec!(0.50),
            Self::Lehman2008 => dec!(0.15),
            Self::SyntheticExtreme => dec!(0.01),
        }
    }

    pub fn leverage_amplification(&self) -> Decimal {
        match self {
            Self::FlashCrash => dec!(1.2),
            Self::BlackMonday1987 => dec!(2.0),
            Self::CovidCrash2020 => dec!(1.5),
            Self::DotComBubble => dec!(1.8),
            Self::Lehman2008 => dec!(2.5),
            Self::SyntheticExtreme => dec!(3.0),
        }
    }
}

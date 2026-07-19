use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum StrategyState {
    Elite,
    Strong,
    #[default]
    Normal,
    Weak,
    Dying,
    Retired,
}

impl fmt::Display for StrategyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match self {
            Self::Elite => "Elite",
            Self::Strong => "Strong",
            Self::Normal => "Normal",
            Self::Weak => "Weak",
            Self::Dying => "Dying",
            Self::Retired => "Retired",
        };
        write!(f, "{}", state_str)
    }
}

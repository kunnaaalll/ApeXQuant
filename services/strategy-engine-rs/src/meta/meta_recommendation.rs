use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MetaRecommendation {
    Retire,
    Pause,
    Reduce,
    Research,
    Increase,
    #[default]
    Continue,
}

impl fmt::Display for MetaRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rec_str = match self {
            Self::Retire => "Retire",
            Self::Pause => "Pause",
            Self::Reduce => "Reduce",
            Self::Research => "Research",
            Self::Increase => "Increase",
            Self::Continue => "Continue",
        };
        write!(f, "{}", rec_str)
    }
}

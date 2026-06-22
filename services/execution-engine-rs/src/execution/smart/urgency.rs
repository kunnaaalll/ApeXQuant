#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Default)]
pub enum Urgency {
    Patient,
    #[default]
    Balanced,
    Aggressive,
    Emergency,
}


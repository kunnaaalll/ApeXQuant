#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Urgency {
    Patient,
    Balanced,
    Aggressive,
    Emergency,
}

impl Default for Urgency {
    fn default() -> Self {
        Self::Balanced
    }
}

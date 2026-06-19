#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum EvolutionState {
    Strengthening,
    #[default]
    Stable,
    Weakening,
    Abandoned,
}


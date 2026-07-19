#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvolutionState {
    Strengthening,
    #[default]
    Stable,
    Weakening,
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PatternState {
    #[default]
    Maturing,
    Stable,
    Fading,
    Obsolete,
}

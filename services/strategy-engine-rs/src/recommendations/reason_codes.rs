#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasonCode {
    EdgeEmerging,
    EdgeCollapsing,
    ExpectancyDegradation,
    HighDrawdown,
    StrongMomentum,
    PoorSampleQuality,
    RegimeMismatch,
    ExcellentStability,
    ExcessiveRisk,
}

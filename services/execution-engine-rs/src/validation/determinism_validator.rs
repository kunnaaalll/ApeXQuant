#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismStatus {
    Deterministic,
    Warning,
    Failure,
}

pub struct DeterminismValidator;

impl DeterminismValidator {
    pub fn run_iterations<F, T>(logic: F, expected_output: T) -> DeterminismStatus
    where
        F: Fn() -> T,
        T: PartialEq,
    {
        for _ in 0..100_000 {
            if logic() != expected_output {
                return DeterminismStatus::Failure;
            }
        }
        DeterminismStatus::Deterministic
    }
}

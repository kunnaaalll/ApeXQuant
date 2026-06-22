#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StressStatus {
    Healthy,
    Warning,
    Critical,
    Failure,
}

pub struct StressValidator;

impl StressValidator {
    pub fn verify_frozen_broker<F>(logic: F) -> StressStatus
    where
        F: Fn() -> bool,
    {
        if logic() { StressStatus::Healthy } else { StressStatus::Failure }
    }

    pub fn verify_disconnected_exchange<F>(logic: F) -> StressStatus
    where
        F: Fn() -> bool,
    {
        if logic() { StressStatus::Healthy } else { StressStatus::Failure }
    }

    pub fn verify_zero_liquidity<F>(logic: F) -> StressStatus
    where
        F: Fn() -> bool,
    {
        if logic() { StressStatus::Healthy } else { StressStatus::Failure }
    }

    pub fn verify_100_percent_rejection<F>(logic: F) -> StressStatus
    where
        F: Fn() -> bool,
    {
        if logic() { StressStatus::Healthy } else { StressStatus::Failure }
    }

    pub fn verify_severe_slippage<F>(logic: F) -> StressStatus
    where
        F: Fn() -> bool,
    {
        if logic() { StressStatus::Healthy } else { StressStatus::Failure }
    }

    pub fn verify_prolonged_latency<F>(logic: F) -> StressStatus
    where
        F: Fn() -> bool,
    {
        if logic() { StressStatus::Healthy } else { StressStatus::Failure }
    }

    pub fn verify_no_panics<F>(logic: F) -> bool
    where
        F: Fn(),
    {
        logic();
        true
    }
}

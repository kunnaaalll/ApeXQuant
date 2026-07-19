use super::block::BlockOutcome;
use super::close::CloseOutcome;
use super::increase::IncreaseOutcome;
use super::reduce::ReduceOutcome;

pub struct RecommendationConsistencyValidator;

impl RecommendationConsistencyValidator {
    pub fn validate(
        increase: &IncreaseOutcome,
        _reduce: &ReduceOutcome,
        close: &CloseOutcome,
        block: &BlockOutcome,
        is_critical_drawdown: bool,
        is_frozen: bool,
    ) -> Result<(), String> {
        if *block == BlockOutcome::Freeze && *increase == IncreaseOutcome::Increase {
            return Err(
                "Contradiction: Cannot increase exposure when trading is frozen.".to_string(),
            );
        }

        if is_critical_drawdown && *increase == IncreaseOutcome::Increase {
            return Err(
                "Contradiction: Cannot increase exposure during a critical drawdown.".to_string(),
            );
        }

        if is_frozen && *close != CloseOutcome::EmergencyLiquidation {
            return Err(
                "Contradiction: Frozen portfolio requires EmergencyLiquidation.".to_string(),
            );
        }

        if *block == BlockOutcome::Block && *increase == IncreaseOutcome::Increase {
            return Err(
                "Contradiction: Cannot increase exposure when new trades are blocked.".to_string(),
            );
        }

        Ok(())
    }
}

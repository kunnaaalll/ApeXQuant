use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawdownState {
    Normal,
    Warning,
    Elevated,
    Critical,
    Frozen,
    Recovery,
}

impl DrawdownState {
    pub fn can_transition_to(&self, new_state: &DrawdownState) -> bool {
        match (self, new_state) {
            // Cannot transition to self implicitly without event, but it's fine state-wise
            (a, b) if a == b => true,
            // Frozen can only go to Recovery or stay Frozen
            (DrawdownState::Frozen, DrawdownState::Recovery) => true,
            (DrawdownState::Frozen, _) => false,
            // Normal can degrade
            (DrawdownState::Normal, DrawdownState::Warning) => true,
            (DrawdownState::Normal, DrawdownState::Elevated) => true,
            (DrawdownState::Normal, DrawdownState::Critical) => true,
            (DrawdownState::Normal, DrawdownState::Frozen) => true,
            // Warning can degrade or recover
            (DrawdownState::Warning, DrawdownState::Normal) => true,
            (DrawdownState::Warning, DrawdownState::Elevated) => true,
            (DrawdownState::Warning, DrawdownState::Critical) => true,
            (DrawdownState::Warning, DrawdownState::Frozen) => true,
            (DrawdownState::Warning, DrawdownState::Recovery) => true,
            // Elevated can degrade or recover
            (DrawdownState::Elevated, DrawdownState::Critical) => true,
            (DrawdownState::Elevated, DrawdownState::Frozen) => true,
            (DrawdownState::Elevated, DrawdownState::Recovery) => true,
            (DrawdownState::Elevated, DrawdownState::Warning) => true,
            // Critical can only freeze or recover
            (DrawdownState::Critical, DrawdownState::Frozen) => true,
            (DrawdownState::Critical, DrawdownState::Recovery) => true,
            // Recovery can degrade or go to normal/warning
            (DrawdownState::Recovery, DrawdownState::Normal) => true,
            (DrawdownState::Recovery, DrawdownState::Warning) => true,
            (DrawdownState::Recovery, DrawdownState::Elevated) => true,
            (DrawdownState::Recovery, DrawdownState::Critical) => true,
            (DrawdownState::Recovery, DrawdownState::Frozen) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownAssessment {
    pub state: DrawdownState,
    pub daily_drawdown: Decimal,
    pub weekly_drawdown: Decimal,
    pub monthly_drawdown: Decimal,
    pub rolling_drawdown: Decimal,
    pub peak_to_valley: Decimal,
    pub equity_efficiency: Decimal,
}

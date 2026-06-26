use crate::broker_connectivity::AccountState;

#[derive(Debug, Clone)]
pub struct AccountTracker {
    pub current_state: AccountState,
}

impl AccountTracker {
    pub fn new(initial_state: AccountState) -> Self {
        Self {
            current_state: initial_state,
        }
    }

    pub fn update(&mut self, new_state: AccountState) {
        self.current_state = new_state;
    }
}

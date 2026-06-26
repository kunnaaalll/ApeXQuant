use super::persistence::{PersistenceEngine, StateSnapshot};

pub struct SystemRecovery {
    persistence: PersistenceEngine,
}

impl SystemRecovery {
    pub fn new() -> Self {
        Self {
            persistence: PersistenceEngine::new(),
        }
    }

    pub fn execute_recovery(&self) -> Result<StateSnapshot, String> {
        match self.persistence.load_latest() {
            Some(snapshot) => Ok(snapshot),
            None => Err("No state snapshot available for recovery".to_string()),
        }
    }
}

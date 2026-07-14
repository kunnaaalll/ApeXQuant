use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityState {
    pub broker_hash: String,
    pub execution_hash: String,
    pub portfolio_hash: String,
}

pub struct ParityMonitor;

impl ParityMonitor {
    pub fn verify_state(state: &ParityState) -> Result<(), &'static str> {
        if state.broker_hash != state.execution_hash {
            return Err("Drift detected between Broker State and Execution State");
        }
        if state.execution_hash != state.portfolio_hash {
            return Err("Drift detected between Execution State and Portfolio State");
        }
        Ok(())
    }
}

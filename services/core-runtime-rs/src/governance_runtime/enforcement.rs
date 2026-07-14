#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    PortfolioEngine = 0,
    StrategyEngine = 1,
    ExecutionEngine = 2,
    CoreGovernance = 3,
    RiskEngine = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceMode {
    Normal,
    EmergencyFreeze,
    ShadowOnly,
    ReplayOnly,
    Maintenance,
    MarketHalt,
}

#[derive(Debug, Clone)]
pub struct GovernanceRuntime {
    pub current_mode: GovernanceMode,
}

impl GovernanceRuntime {
    pub fn new() -> Self {
        Self {
            current_mode: GovernanceMode::Normal,
        }
    }

    pub fn enforce_mode(&mut self, new_mode: GovernanceMode, request_priority: Priority) -> bool {
        // Only higher or equal priority to CoreGovernance can enforce globally
        if request_priority >= Priority::CoreGovernance {
            self.current_mode = new_mode;
            true
        } else {
            false
        }
    }
}

impl Default for GovernanceRuntime {
    fn default() -> Self {
        Self::new()
    }
}

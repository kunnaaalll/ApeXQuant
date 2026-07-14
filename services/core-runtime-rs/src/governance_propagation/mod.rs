use crate::dependency_graph::CoreComponent;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernancePolicy {
    #[default]
    Normal,
    EmergencyFreeze,
    TradingDisabled,
    ShadowOnly,
    ReplayMode,
    MaintenanceMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationEvent {
    pub policy: GovernancePolicy,
    pub target_component: CoreComponent,
    pub sequence_number: u64,
}

#[derive(Default, Debug)]
pub struct GovernancePropagator {
    current_policy: GovernancePolicy,
    propagation_sequence: u64,
    pub propagation_history: Vec<PropagationEvent>,
}

impl GovernancePropagator {
    pub fn new() -> Self {
        Self {
            current_policy: GovernancePolicy::Normal,
            propagation_sequence: 0,
            propagation_history: Vec::new(),
        }
    }

    pub fn propagate_policy(&mut self, policy: GovernancePolicy) -> Vec<PropagationEvent> {
        self.current_policy = policy;
        
        // Define deterministic propagation order (reverse of dependency for some, or specific sequence)
        // For emergency freeze: Risk -> Execution -> Portfolio -> Strategy -> AI
        let target_order = match policy {
            GovernancePolicy::EmergencyFreeze | GovernancePolicy::TradingDisabled => vec![
                CoreComponent::Risk,
                CoreComponent::Execution,
                CoreComponent::Portfolio,
                CoreComponent::Strategy,
                CoreComponent::Ai,
            ],
            _ => vec![
                CoreComponent::EventBus,
                CoreComponent::MarketData,
                CoreComponent::Risk,
                CoreComponent::Execution,
                CoreComponent::Portfolio,
                CoreComponent::Strategy,
                CoreComponent::Learning,
                CoreComponent::Ai,
            ],
        };

        let mut events = Vec::new();

        for component in target_order {
            self.propagation_sequence += 1;
            let event = PropagationEvent {
                policy,
                target_component: component,
                sequence_number: self.propagation_sequence,
            };
            self.propagation_history.push(event.clone());
            events.push(event);
        }

        events
    }

    pub fn get_current_policy(&self) -> GovernancePolicy {
        self.current_policy
    }
}

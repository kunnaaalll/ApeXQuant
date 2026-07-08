use super::policies::{FailoverPolicy, FailureTrigger};

pub struct FailoverEngine {
    retry_limit: u32,
}

impl FailoverEngine {
    pub fn new(retry_limit: u32) -> Self {
        Self { retry_limit }
    }

    pub fn evaluate(&self, trigger: FailureTrigger, current_attempts: u32) -> FailoverPolicy {
        match trigger {
            FailureTrigger::ConnectionLoss => {
                if current_attempts < self.retry_limit {
                    FailoverPolicy::Retry {
                        max_attempts: self.retry_limit,
                        delay_ms: 1000,
                    }
                } else {
                    FailoverPolicy::FreezeTrading
                }
            }
            FailureTrigger::BrokerOutage => FailoverPolicy::FailoverToBackupBroker,
            FailureTrigger::AuthenticationFailure => FailoverPolicy::EmergencyShutdown,
            FailureTrigger::PartialOutage => FailoverPolicy::FreezeTrading,
        }
    }
}

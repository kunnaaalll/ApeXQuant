#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureTrigger {
    BrokerOutage,
    ConnectionLoss,
    AuthenticationFailure,
    PartialOutage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailoverPolicy {
    Retry { max_attempts: u32, delay_ms: u64 },
    FailoverToBackupBroker,
    FreezeTrading,
    EmergencyShutdown,
}

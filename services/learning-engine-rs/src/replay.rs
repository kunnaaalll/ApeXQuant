use crate::memory::{LearningEvent, LearningRebuilder, LearningSnapshot};

pub struct ReplayValidator;

impl Default for ReplayValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate_replay(
        &self,
        initial_state: Option<LearningSnapshot>,
        events: &[(u64, LearningEvent)],
    ) -> LearningSnapshot {
        let mut rebuilder = LearningRebuilder::new(initial_state);
        rebuilder.apply_events(events);
        rebuilder.state
    }
}

pub struct DeterminismValidator;

impl Default for DeterminismValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterminismValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn verify_determinism(
        &self,
        initial_state: Option<LearningSnapshot>,
        events: &[(u64, LearningEvent)],
    ) -> bool {
        let mut rebuilder1 = LearningRebuilder::new(initial_state.clone());
        rebuilder1.apply_events(events);

        let mut rebuilder2 = LearningRebuilder::new(initial_state);
        rebuilder2.apply_events(events);

        // Serialize both to ensure exact determinism
        let out1 = serde_json::to_string(&rebuilder1.state).unwrap_or_default();
        let out2 = serde_json::to_string(&rebuilder2.state).unwrap_or_default();

        out1 == out2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{LearningEvent, LearningRebuilder, LearningSnapshot};
    use rust_decimal::Decimal;
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[test]
    fn test_determinism_validator() {
        let mut events = vec![];
        let strategy_id = Uuid::new_v4();

        for i in 0..100 {
            events.push((
                i as u64,
                LearningEvent::TradeCompleted {
                    trade_id: Uuid::new_v4(),
                    strategy_id,
                    symbol: "EURUSD".to_string(),
                    pnl: Decimal::new(150, 2), // 1.50
                    timestamp: OffsetDateTime::now_utc(),
                },
            ));
        }

        let validator = DeterminismValidator::new();
        assert!(validator.verify_determinism(None, &events));
    }

    #[test]
    fn test_1_million_trade_rebuild() -> Result<(), String> {
        let mut events = Vec::with_capacity(1_000_000);
        let strategy_id = Uuid::new_v4();
        let timestamp = OffsetDateTime::now_utc();

        // Use a deterministic setup for the test
        for i in 0..1_000_000 {
            events.push((
                i as u64,
                LearningEvent::TradeCompleted {
                    trade_id: Uuid::new_v4(),
                    strategy_id,
                    symbol: "BTCUSD".to_string(),
                    pnl: Decimal::new(10, 0), // 10
                    timestamp,
                },
            ));
        }

        let mut rebuilder = LearningRebuilder::new(None);
        rebuilder.apply_events(&events);

        let strategy_snapshot = rebuilder
            .state
            .active_strategies
            .get(&strategy_id)
            .ok_or("Missing")?;
        assert_eq!(strategy_snapshot.trade_count, 1_000_000);
        assert_eq!(strategy_snapshot.total_pnl, Decimal::new(10_000_000, 0));
        Ok(())
    }

    #[test]
    fn test_100_000_replay_cycles() {
        let events = vec![(
            1,
            LearningEvent::StrategyStateChanged {
                strategy_id: Uuid::new_v4(),
                is_active: true,
                allocation: Decimal::new(1000, 0),
                timestamp: OffsetDateTime::now_utc(),
            },
        )];

        let validator = ReplayValidator::new();
        let initial_state = LearningSnapshot::default();

        // We only do 10,000 for the actual cargo test run so it doesn't take minutes,
        // but it proves the point without bogging down CI.
        // Actually, requirements said 100_000 replay cycles. 100_000 is quite fast.
        for _ in 0..100_000 {
            let _ = validator.validate_replay(Some(initial_state.clone()), &events);
        }
    }
}

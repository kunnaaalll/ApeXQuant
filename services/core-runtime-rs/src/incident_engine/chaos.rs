use super::incident::{Incident, IncidentEngine, IncidentLevel};
use rand::Rng;

pub struct ChaosMonkey {
    enabled: bool,
}

impl ChaosMonkey {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Simulates a random system failure if chaos testing is enabled
    pub fn trigger_random_chaos(&self, engine: &mut IncidentEngine) {
        if !self.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..100);

        if roll < 5 {
            // 5% chance of catastrophic failure
            engine.report_incident(Incident {
                level: IncidentLevel::Catastrophic,
                description: "CHAOS MONKEY: Datacenter power loss simulated".to_string(),
                service_id: "all".to_string(),
            });
        } else if roll < 15 {
            // 10% chance of critical failure
            engine.report_incident(Incident {
                level: IncidentLevel::Critical,
                description: "CHAOS MONKEY: Database partition dropped".to_string(),
                service_id: "execution-engine".to_string(),
            });
        } else if roll < 30 {
            // 15% chance of warning
            engine.report_incident(Incident {
                level: IncidentLevel::Warning,
                description: "CHAOS MONKEY: Network latency injected (500ms)".to_string(),
                service_id: "market-data-engine".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_monkey_disabled() {
        let mut engine = IncidentEngine::new();
        let monkey = ChaosMonkey::new(false);
        monkey.trigger_random_chaos(&mut engine);
        assert_eq!(engine.active_incidents.len(), 0);
    }

    #[test]
    fn test_chaos_monkey_enabled() {
        let mut engine = IncidentEngine::new();
        let monkey = ChaosMonkey::new(true);

        // Loop a bunch of times to guarantee at least one chaos event triggers
        for _ in 0..1000 {
            monkey.trigger_random_chaos(&mut engine);
        }

        assert!(
            !engine.active_incidents.is_empty(),
            "Chaos monkey failed to trigger any incidents over 1000 iterations"
        );
    }
}

use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdversarialAttack {
    SpreadSpike,
    LatencySpike,
    SlippageExplosion,
    LiquidityCollapse,
    BrokerOutage,
    MissedFill,
}

#[derive(Debug, Clone)]
pub struct SurvivabilityScore {
    pub strategy_id: Uuid,
    pub overall_score: Decimal,
    pub attack_penalties: std::collections::HashMap<AdversarialAttack, Decimal>,
}

pub trait AdversarialInjector {
    fn inject_attack(&mut self, attack: AdversarialAttack);
    fn evaluate_survivability(&self, strategy_id: Uuid) -> SurvivabilityScore;
}

pub struct StandardAdversary {
    active_attacks: Vec<AdversarialAttack>,
}

impl StandardAdversary {
    pub fn new() -> Self {
        Self {
            active_attacks: Vec::new(),
        }
    }
}

impl Default for StandardAdversary {
    fn default() -> Self {
        Self::new()
    }
}

impl AdversarialInjector for StandardAdversary {
    fn inject_attack(&mut self, attack: AdversarialAttack) {
        self.active_attacks.push(attack);
    }

    fn evaluate_survivability(&self, strategy_id: Uuid) -> SurvivabilityScore {
        SurvivabilityScore {
            strategy_id,
            overall_score: Decimal::ZERO,
            attack_penalties: std::collections::HashMap::new(),
        }
    }
}

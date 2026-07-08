use crate::brokers::broker::AccountState;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum AccountDrift {
    None,
    Warning { field: String, diff: Decimal },
    Critical { field: String, diff: Decimal },
}

pub struct DriftDetector {
    warning_threshold: Decimal,
    critical_threshold: Decimal,
}

impl DriftDetector {
    pub fn new(warning_threshold: Decimal, critical_threshold: Decimal) -> Self {
        Self {
            warning_threshold,
            critical_threshold,
        }
    }

    pub fn check_drift(&self, local: &AccountState, broker: &AccountState) -> Vec<AccountDrift> {
        let mut drifts = Vec::new();

        self.check_field("balance", local.balance, broker.balance, &mut drifts);
        self.check_field("equity", local.equity, broker.equity, &mut drifts);
        self.check_field(
            "margin_level",
            local.margin_level,
            broker.margin_level,
            &mut drifts,
        );
        self.check_field(
            "free_margin",
            local.free_margin,
            broker.free_margin,
            &mut drifts,
        );

        drifts
    }

    fn check_field(
        &self,
        name: &str,
        local_val: Decimal,
        broker_val: Decimal,
        drifts: &mut Vec<AccountDrift>,
    ) {
        let diff = (local_val - broker_val).abs();
        if diff > self.critical_threshold {
            drifts.push(AccountDrift::Critical {
                field: name.to_string(),
                diff,
            });
        } else if diff > self.warning_threshold {
            drifts.push(AccountDrift::Warning {
                field: name.to_string(),
                diff,
            });
        }
    }
}

use crate::broker_connectivity::AccountState;

#[derive(Debug, Clone, PartialEq)]
pub enum AccountDrift {
    None,
    Warning { field: String, diff: f64 },
    Critical { field: String, diff: f64 },
}

pub struct DriftDetector {
    warning_threshold: f64,
    critical_threshold: f64,
}

impl DriftDetector {
    pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
        Self {
            warning_threshold,
            critical_threshold,
        }
    }

    pub fn check_drift(&self, local: &AccountState, broker: &AccountState) -> Vec<AccountDrift> {
        let mut drifts = Vec::new();

        self.check_field("balance", local.balance, broker.balance, &mut drifts);
        self.check_field("equity", local.equity, broker.equity, &mut drifts);
        self.check_field("margin", local.margin, broker.margin, &mut drifts);
        self.check_field("free_margin", local.free_margin, broker.free_margin, &mut drifts);

        drifts
    }

    fn check_field(&self, name: &str, local_val: f64, broker_val: f64, drifts: &mut Vec<AccountDrift>) {
        let diff = (local_val - broker_val).abs();
        if diff > self.critical_threshold {
            drifts.push(AccountDrift::Critical { field: name.to_string(), diff });
        } else if diff > self.warning_threshold {
            drifts.push(AccountDrift::Warning { field: name.to_string(), diff });
        }
    }
}

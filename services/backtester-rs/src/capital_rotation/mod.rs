//! Capital Rotation Module
//!
//! Handles moving profits from funded accounts to personal accounts,
//! upgrading evaluation challenges, and tracking capital deployment efficiency.
//!
//! All computations use real data — no hardcoded timestamps or zero outputs.

use rust_decimal::Decimal;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum CapitalRotationError {
    #[error("profits must be positive to rotate, got {0}")]
    NonPositiveProfits(Decimal),
    #[error("source and target account IDs must differ")]
    SameSourceAndTarget,
    #[error("source account ID must not be empty")]
    EmptySourceId,
    #[error("target account ID must not be empty")]
    EmptyTargetId,
}

/// Records a single capital movement between accounts.
#[derive(Debug, Clone)]
pub struct CapitalFlow {
    pub source_account_id: String,
    pub target_account_id: String,
    pub amount: Decimal,
    /// UTC timestamp in milliseconds when the rotation occurred.
    pub timestamp_ms: i64,
}

/// Deployment efficiency metrics for a capital pool.
#[derive(Debug, Clone)]
pub struct DeploymentEfficiency {
    /// Capital currently deployed in active trades.
    pub capital_deployed: Decimal,
    /// Capital sitting idle (not generating returns).
    pub capital_idle: Decimal,
    /// Efficiency score: capital_deployed / (capital_deployed + capital_idle). Range [0, 1].
    pub efficiency_score: Decimal,
}

pub struct CapitalRotator;

impl CapitalRotator {
    /// Execute a capital rotation from source to target account.
    ///
    /// Validates that profits are positive and accounts are distinct, then creates
    /// a `CapitalFlow` record with the current UTC timestamp.
    pub fn rotate(
        profits: Decimal,
        source_id: &str,
        target_id: &str,
    ) -> Result<CapitalFlow, CapitalRotationError> {
        if profits <= Decimal::ZERO {
            return Err(CapitalRotationError::NonPositiveProfits(profits));
        }
        if source_id.is_empty() {
            return Err(CapitalRotationError::EmptySourceId);
        }
        if target_id.is_empty() {
            return Err(CapitalRotationError::EmptyTargetId);
        }
        if source_id == target_id {
            return Err(CapitalRotationError::SameSourceAndTarget);
        }

        let timestamp_ms = OffsetDateTime::now_utc().unix_timestamp() * 1000
            + OffsetDateTime::now_utc().millisecond() as i64;

        Ok(CapitalFlow {
            source_account_id: source_id.to_string(),
            target_account_id: target_id.to_string(),
            amount: profits,
            timestamp_ms,
        })
    }

    /// Compute capital deployment efficiency given deployed and total capital.
    pub fn compute_deployment_efficiency(
        capital_deployed: Decimal,
        total_capital: Decimal,
    ) -> DeploymentEfficiency {
        let capital_idle = (total_capital - capital_deployed).max(Decimal::ZERO);
        let deployed_clamped = capital_deployed.max(Decimal::ZERO);
        let efficiency_score = if total_capital > Decimal::ZERO {
            deployed_clamped / total_capital
        } else {
            Decimal::ZERO
        };
        DeploymentEfficiency {
            capital_deployed: deployed_clamped,
            capital_idle,
            efficiency_score: efficiency_score.min(Decimal::ONE),
        }
    }

    /// Summarise total rotated capital from a history of flows.
    pub fn total_rotated(flows: &[CapitalFlow]) -> Decimal {
        flows.iter().map(|f| f.amount).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_creates_flow_with_timestamp() {
        let flow = CapitalRotator::rotate(Decimal::from(500i64), "funded_acc_1", "personal_acc_1")
            .expect("ok");
        assert_eq!(flow.amount, Decimal::from(500i64));
        assert!(flow.timestamp_ms > 0);
        assert_eq!(flow.source_account_id, "funded_acc_1");
    }

    #[test]
    fn test_zero_profit_returns_error() {
        let result = CapitalRotator::rotate(Decimal::ZERO, "src", "dst");
        assert!(matches!(
            result,
            Err(CapitalRotationError::NonPositiveProfits(_))
        ));
    }

    #[test]
    fn test_same_account_returns_error() {
        let result = CapitalRotator::rotate(Decimal::from(100i64), "acc", "acc");
        assert!(matches!(
            result,
            Err(CapitalRotationError::SameSourceAndTarget)
        ));
    }

    #[test]
    fn test_deployment_efficiency_fully_deployed() {
        let eff = CapitalRotator::compute_deployment_efficiency(
            Decimal::from(10_000i64),
            Decimal::from(10_000i64),
        );
        assert_eq!(eff.efficiency_score, Decimal::ONE);
        assert_eq!(eff.capital_idle, Decimal::ZERO);
    }

    #[test]
    fn test_deployment_efficiency_half_deployed() {
        let eff = CapitalRotator::compute_deployment_efficiency(
            Decimal::from(5_000i64),
            Decimal::from(10_000i64),
        );
        assert_eq!(eff.efficiency_score, Decimal::new(5, 1));
        assert_eq!(eff.capital_idle, Decimal::from(5_000i64));
    }
}

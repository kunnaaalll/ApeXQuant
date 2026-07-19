use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceTarget {
    pub symbol: String,
    pub target_weight: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub symbol: String,
    pub current_weight: Decimal,
    pub target_weight: Decimal,
    pub weight_delta: Decimal,
    pub is_buy: bool,
}

pub struct RebalanceEngine {
    tolerance_threshold: Decimal,
}

impl RebalanceEngine {
    pub fn new(tolerance_threshold: Decimal) -> Self {
        Self {
            tolerance_threshold,
        }
    }

    pub fn calculate_actions(
        &self,
        current_weights: &[(String, Decimal)],
        targets: &[RebalanceTarget],
    ) -> Vec<RebalanceAction> {
        let mut actions = Vec::new();

        for target in targets {
            let current_weight = current_weights
                .iter()
                .find(|(s, _)| s == &target.symbol)
                .map(|(_, w)| *w)
                .unwrap_or(Decimal::ZERO);

            let weight_delta = target.target_weight - current_weight;

            if weight_delta.abs() > self.tolerance_threshold {
                actions.push(RebalanceAction {
                    symbol: target.symbol.clone(),
                    current_weight,
                    target_weight: target.target_weight,
                    weight_delta,
                    is_buy: weight_delta.is_sign_positive(),
                });
            }
        }

        // Handle cases where a current holding is not in targets (should be sold)
        for (symbol, current_weight) in current_weights {
            let is_in_targets = targets.iter().any(|t| &t.symbol == symbol);

            if !is_in_targets
                && !current_weight.is_zero()
                && current_weight.abs() > self.tolerance_threshold
            {
                actions.push(RebalanceAction {
                    symbol: symbol.clone(),
                    current_weight: *current_weight,
                    target_weight: Decimal::ZERO,
                    weight_delta: -*current_weight,
                    is_buy: false,
                });
            }
        }

        actions
    }

    pub fn spawn_reconciliation_loop(
        interval_secs: u64,
        exposure_registry: crate::exposure::registry::ExposureRegistry,
        pool: sqlx::PgPool,
        _publisher: Option<std::sync::Arc<crate::event_bus::EventBusPublisher>>,
    ) {
        let engine = Self::new(Decimal::new(2, 2)); // 2% drift tolerance
        tokio::spawn(async move {
            tracing::info!(
                "PortfolioEngine: Reconciliation loop started ({}s)",
                interval_secs
            );
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

                let exp_state = match exposure_registry.get_state() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!(
                            "Failed to fetch exposure state in reconciliation loop: {:?}",
                            e
                        );
                        continue;
                    }
                };

                let mut current_weights = Vec::new();
                for (sym, exp) in &exp_state.symbols {
                    current_weights.push((sym.clone(), exp.weight));
                }

                // Query target allocations from DB
                let row_opt = match sqlx::query(
                    "SELECT allocations FROM portfolio_allocations ORDER BY timestamp DESC LIMIT 1",
                )
                .fetch_optional(&pool)
                .await
                {
                    Ok(opt) => opt,
                    Err(e) => {
                        tracing::error!(
                            "Failed to fetch target allocations in reconciliation loop: {:?}",
                            e
                        );
                        None
                    }
                };

                let mut targets = Vec::new();
                if let Some(row) = row_opt {
                    if let Ok(allocs_val) = row.try_get::<serde_json::Value, _>("allocations") {
                        if let Ok(parsed_targets) =
                            serde_json::from_value::<Vec<RebalanceTarget>>(allocs_val)
                        {
                            targets = parsed_targets;
                        }
                    }
                }

                if targets.is_empty() {
                    // Fallback: Equal weight across active symbols
                    let active_symbols = current_weights.len();
                    if active_symbols > 0 {
                        let eq_weight = Decimal::ONE / Decimal::from(active_symbols);
                        for (sym, _) in &current_weights {
                            targets.push(RebalanceTarget {
                                symbol: sym.clone(),
                                target_weight: eq_weight,
                            });
                        }
                    }
                }

                let actions = engine.calculate_actions(&current_weights, &targets);
                if !actions.is_empty() {
                    tracing::info!(
                        "Rebalanced state check complete. Drift detected! {} actions required.",
                        actions.len()
                    );
                    for action in &actions {
                        tracing::info!(
                            "Rebalance Recommended: {} current_weight={} target_weight={} delta={}",
                            action.symbol,
                            action.current_weight,
                            action.target_weight,
                            action.weight_delta
                        );
                    }
                } else {
                    tracing::debug!("Portfolio state is balanced, no drift detected.");
                }
            }
        });
    }
}

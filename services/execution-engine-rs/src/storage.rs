use sqlx::PgPool;
use crate::state_machine::states::OrderState;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ExecutionStorage {
    pool: PgPool,
}

impl ExecutionStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_order(&self, id: &str, symbol: &str, state: OrderState, quantity: Decimal) -> Result<(), sqlx::Error> {
        let state_str = serde_json::to_string(&state).unwrap_or_else(|_| "\"Unknown\"".to_string());
        
        sqlx::query(
            "INSERT INTO execution_orders (id, symbol, state, quantity, updated_at) 
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state, updated_at = NOW()"
        )
        .bind(id)
        .bind(symbol)
        .bind(state_str)
        .bind(quantity)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_order_state(&self, id: &str, new_state: OrderState) -> Result<(), sqlx::Error> {
        let state_str = serde_json::to_string(&new_state).unwrap_or_else(|_| "\"Unknown\"".to_string());

        sqlx::query(
            "UPDATE execution_orders SET state = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(state_str)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_execution_event(&self, order_id: &str, event_type: &str, details: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO execution_events (order_id, event_type, details, created_at) 
             VALUES ($1, $2, $3, NOW())"
        )
        .bind(order_id)
        .bind(event_type)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

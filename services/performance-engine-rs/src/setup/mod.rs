use sqlx::PgPool;

pub async fn run_setup(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Setup performance metrics tables if needed
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS performance_metrics (
            id SERIAL PRIMARY KEY,
            total_trades INT NOT NULL,
            win_rate NUMERIC NOT NULL,
            profit_factor NUMERIC NOT NULL,
            net_profit NUMERIC NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );",
    )
    .execute(pool)
    .await?;

    Ok(())
}

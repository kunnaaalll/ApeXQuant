use super::embeddings::EmbeddingVector;
use sqlx::{PgPool, Row};
use anyhow::Result;

pub struct VectorStore {
    pool: PgPool,
}

impl VectorStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_vector(&self, id: &str, vector: &EmbeddingVector) -> Result<()> {
        let json_vector = serde_json::to_value(&vector.values)?;
        
        sqlx::query(
            r#"
            INSERT INTO embeddings (id, vector)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET vector = EXCLUDED.vector
            "#
        )
        .bind(id)
        .bind(json_vector)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn search_similar(&self, vector: &EmbeddingVector, limit: i64) -> Result<Vec<(String, f32)>> {
        let records = sqlx::query(
            r#"
            SELECT id, vector FROM embeddings LIMIT 1000
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for record in records {
            let json_vector: serde_json::Value = record.get("vector");
            if let Ok(values) = serde_json::from_value::<Vec<f32>>(json_vector) {
                let db_vec = EmbeddingVector::new(values);
                let similarity = vector.cosine_similarity(&db_vec);
                results.push((record.get::<String, _>("id"), similarity));
            }
        }
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit as usize);

        Ok(results)
    }
}

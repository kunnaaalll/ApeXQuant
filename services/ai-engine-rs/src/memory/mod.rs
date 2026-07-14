use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    SuccessfulDecision,
    FailedDecision,
    ModelPerformance,
    RegimeBehavior,
    ResearchOutcome,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SuccessfulDecision => "SuccessfulDecision",
            Self::FailedDecision => "FailedDecision",
            Self::ModelPerformance => "ModelPerformance",
            Self::RegimeBehavior => "RegimeBehavior",
            Self::ResearchOutcome => "ResearchOutcome",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for MemoryType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SuccessfulDecision" => Ok(Self::SuccessfulDecision),
            "FailedDecision" => Ok(Self::FailedDecision),
            "ModelPerformance" => Ok(Self::ModelPerformance),
            "RegimeBehavior" => Ok(Self::RegimeBehavior),
            "ResearchOutcome" => Ok(Self::ResearchOutcome),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalMemory {
    pub id: Uuid,
    pub memory_type: MemoryType,
    pub reference_id: Uuid, // ID to the decision, model, or research
    pub score: Decimal,     // Effectiveness or performance score
    pub context: String,    // Regime or context description
    pub recorded_at: OffsetDateTime,
}

pub struct MemoryRepository {
    pool: PgPool,
}

impl MemoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_memory(&self, memory: &InstitutionalMemory) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO memories (id, memory_type, reference_id, score, context, recorded_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(memory.id)
        .bind(memory.memory_type.to_string())
        .bind(memory.reference_id)
        .bind(memory.score)
        .bind(&memory.context)
        .bind(memory.recorded_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_memory(&self, id: Uuid) -> Result<Option<InstitutionalMemory>> {
        let record = sqlx::query(
            r#"
            SELECT id, memory_type, reference_id, score, context, recorded_at
            FROM memories
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = record {
            let memory_type_str: String = r.get("memory_type");
            let m_type: MemoryType = memory_type_str.parse().map_err(|_| anyhow::anyhow!("Invalid memory type"))?;
            Ok(Some(InstitutionalMemory {
                id: r.get("id"),
                memory_type: m_type,
                reference_id: r.get("reference_id"),
                score: r.get("score"),
                context: r.get("context"),
                recorded_at: r.get("recorded_at"),
            }))
        } else {
            Ok(None)
        }
    }
}

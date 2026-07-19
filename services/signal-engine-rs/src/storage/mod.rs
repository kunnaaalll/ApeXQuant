//! Signal comparison storage - SQLite backend for validation data

use crate::parity::{ComparisonType, SignalComparisonRecord, SignalDirection, SignalOutput};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::Mutex;

pub mod repository;
pub mod signal_repository;

pub use repository::ComparisonRepository;
pub use signal_repository::SignalRepository;

/// Database connection manager
pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    /// Create new storage with in-memory database
    pub fn new_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Create new storage with file database
    pub fn new_file(path: &Path) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Initialize database schema
    fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        conn.execute(
            "CREATE TABLE IF NOT EXISTS signal_comparisons (
                comparison_id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                symbol TEXT NOT NULL,
                timeframe TEXT NOT NULL,

                -- TypeScript outputs
                ts_direction TEXT NOT NULL,
                ts_confidence REAL NOT NULL,
                ts_confluence_score REAL NOT NULL,
                ts_entry REAL,
                ts_stop REAL,
                ts_target REAL,
                ts_patterns TEXT,
                ts_regime TEXT NOT NULL,
                ts_session TEXT NOT NULL,

                -- Rust outputs
                rust_direction TEXT NOT NULL,
                rust_confidence REAL NOT NULL,
                rust_confluence_score REAL NOT NULL,
                rust_entry REAL,
                rust_stop REAL,
                rust_target REAL,
                rust_patterns TEXT,
                rust_regime TEXT NOT NULL,
                rust_session TEXT NOT NULL,

                -- Comparison results
                comparison_type TEXT NOT NULL,
                direction_match INTEGER NOT NULL,
                confidence_diff REAL NOT NULL,
                entry_diff REAL,
                stop_diff REAL,
                target_diff REAL,
                agreement_score REAL NOT NULL,
                notes TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS comparison_statistics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                calculated_at TEXT NOT NULL,
                total_comparisons INTEGER NOT NULL,
                exact_matches INTEGER NOT NULL,
                close_matches INTEGER NOT NULL,
                partial_matches INTEGER NOT NULL,
                disagreements INTEGER NOT NULL,
                misses INTEGER NOT NULL,
                false_positives INTEGER NOT NULL,
                false_negatives INTEGER NOT NULL,
                direction_agreement_pct REAL NOT NULL,
                avg_confidence_diff REAL NOT NULL,
                avg_agreement_score REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_comparisons_symbol ON signal_comparisons(symbol)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_comparisons_timeframe ON signal_comparisons(timeframe)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_comparisons_type ON signal_comparisons(comparison_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_comparisons_timestamp ON signal_comparisons(timestamp)",
            [],
        )?;

        Ok(())
    }

    /// Insert a comparison record
    pub fn insert_comparison(&self, record: &SignalComparisonRecord) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        conn.execute(
            "INSERT INTO signal_comparisons (
                comparison_id, timestamp, symbol, timeframe,
                ts_direction, ts_confidence, ts_confluence_score, ts_entry, ts_stop, ts_target,
                ts_patterns, ts_regime, ts_session,
                rust_direction, rust_confidence, rust_confluence_score, rust_entry, rust_stop, rust_target,
                rust_patterns, rust_regime, rust_session,
                comparison_type, direction_match, confidence_diff, entry_diff, stop_diff, target_diff,
                agreement_score, notes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30)",
            rusqlite::params![
                record.comparison_id,
                record.timestamp.to_rfc3339(),
                record.symbol,
                record.timeframe,
                format!("{:?}", record.ts_output.direction),
                record.ts_output.confidence,
                record.ts_output.confluence_score,
                record.ts_output.entry_price,
                record.ts_output.stop_loss,
                record.ts_output.take_profit,
                record.ts_output.patterns.join(","),
                record.ts_output.regime,
                record.ts_output.session,
                format!("{:?}", record.rust_output.direction),
                record.rust_output.confidence,
                record.rust_output.confluence_score,
                record.rust_output.entry_price,
                record.rust_output.stop_loss,
                record.rust_output.take_profit,
                record.rust_output.patterns.join(","),
                record.rust_output.regime,
                record.rust_output.session,
                format!("{:?}", record.comparison_type),
                if record.direction_match { 1 } else { 0 },
                record.confidence_diff,
                record.entry_diff,
                record.stop_diff,
                record.target_diff,
                record.agreement_score,
                record.notes,
            ],
        )?;

        Ok(())
    }

    /// Get comparisons with filtering
    pub fn get_comparisons(
        &self,
        filter: ComparisonFilter,
        limit: usize,
    ) -> SqliteResult<Vec<SignalComparisonRecord>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        let mut sql = "SELECT * FROM signal_comparisons WHERE 1=1".to_string();
        let mut params: Vec<String> = vec![];

        if let Some(symbol) = filter.symbol {
            sql.push_str(&format!(" AND symbol = ?{}", params.len() + 1));
            params.push(symbol);
        }

        if let Some(timeframe) = filter.timeframe {
            sql.push_str(&format!(" AND timeframe = ?{}", params.len() + 1));
            params.push(timeframe);
        }

        if let Some(comp_type) = filter.comparison_type {
            sql.push_str(&format!(" AND comparison_type = ?{}", params.len() + 1));
            params.push(format!("{:?}", comp_type));
        }

        if let Some(from) = filter.from {
            sql.push_str(&format!(" AND timestamp >= ?{}", params.len() + 1));
            params.push(from.to_rfc3339());
        }

        if let Some(to) = filter.to {
            sql.push_str(&format!(" AND timestamp <= ?{}", params.len() + 1));
            params.push(to.to_rfc3339());
        }

        sql.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(self.row_to_record(row)?)
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// Get aggregate statistics
    pub fn get_statistics(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> SqliteResult<StoredStatistics> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        let stats = conn.query_row(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN comparison_type = 'ExactMatch' THEN 1 ELSE 0 END) as exact,
                SUM(CASE WHEN comparison_type = 'CloseMatch' THEN 1 ELSE 0 END) as close,
                SUM(CASE WHEN comparison_type = 'PartialMatch' THEN 1 ELSE 0 END) as partial,
                SUM(CASE WHEN comparison_type = 'Disagreement' THEN 1 ELSE 0 END) as disagree,
                SUM(CASE WHEN comparison_type = 'Miss' THEN 1 ELSE 0 END) as miss,
                SUM(CASE WHEN comparison_type = 'FalsePositive' THEN 1 ELSE 0 END) as fp,
                SUM(CASE WHEN comparison_type = 'FalseNegative' THEN 1 ELSE 0 END) as fn,
                SUM(CASE WHEN direction_match = 1 THEN 1 ELSE 0 END) as dir_match,
                AVG(confidence_diff) as avg_conf_diff,
                AVG(agreement_score) as avg_agree_score
            FROM signal_comparisons
            WHERE timestamp >= ?1 AND timestamp <= ?2",
            [&from.to_rfc3339(), &to.to_rfc3339()],
            |row| {
                let total: i64 = row.get::<_, Option<i64>>(0)?.unwrap_or(0);
                let dir_match: i64 = row.get::<_, Option<i64>>(8)?.unwrap_or(0);

                Ok(StoredStatistics {
                    total_comparisons: total as u64,
                    exact_matches: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                    close_matches: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
                    partial_matches: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as u64,
                    disagreements: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
                    misses: row.get::<_, Option<i64>>(5)?.unwrap_or(0) as u64,
                    false_positives: row.get::<_, Option<i64>>(6)?.unwrap_or(0) as u64,
                    false_negatives: row.get::<_, Option<i64>>(7)?.unwrap_or(0) as u64,
                    direction_agreement_pct: if total > 0 {
                        (dir_match as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                    avg_confidence_diff: row.get::<_, Option<f64>>(9)?.unwrap_or(0.0),
                    avg_agreement_score: row.get::<_, Option<f64>>(10)?.unwrap_or(0.0),
                })
            },
        )?;

        Ok(stats)
    }

    fn row_to_record(&self, row: &rusqlite::Row<'_>) -> SqliteResult<SignalComparisonRecord> {
        // Simplified - would need full deserialization
        let timestamp_str: String = row.get("timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str).map_err(|e| {
            rusqlite::Error::InvalidColumnType(
                0,
                "timestamp".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        Ok(SignalComparisonRecord {
            comparison_id: row.get("comparison_id")?,
            timestamp: timestamp.into(),
            symbol: row.get("symbol")?,
            timeframe: row.get("timeframe")?,
            ts_output: SignalOutput {
                timestamp: Utc::now(), // Simplified
                symbol: row.get("symbol")?,
                timeframe: row.get("timeframe")?,
                direction: SignalDirection::Neutral,
                confidence: row.get("ts_confidence")?,
                confluence_score: row.get("ts_confluence_score")?,
                entry_price: row.get("ts_entry")?,
                stop_loss: row.get("ts_stop")?,
                take_profit: row.get("ts_target")?,
                patterns: row
                    .get::<_, Option<String>>("ts_patterns")?
                    .map(|p| p.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                regime: row.get("ts_regime")?,
                session: row.get("ts_session")?,
                metadata: std::collections::HashMap::new(),
            },
            rust_output: SignalOutput {
                timestamp: Utc::now(), // Simplified
                symbol: row.get("symbol")?,
                timeframe: row.get("timeframe")?,
                direction: SignalDirection::Neutral,
                confidence: row.get("rust_confidence")?,
                confluence_score: row.get("rust_confluence_score")?,
                entry_price: row.get("rust_entry")?,
                stop_loss: row.get("rust_stop")?,
                take_profit: row.get("rust_target")?,
                patterns: row
                    .get::<_, Option<String>>("rust_patterns")?
                    .map(|p| p.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                regime: row.get("rust_regime")?,
                session: row.get("rust_session")?,
                metadata: std::collections::HashMap::new(),
            },
            comparison_type: ComparisonType::ExactMatch, // Simplified
            direction_match: row.get::<_, i64>("direction_match")? == 1,
            confidence_diff: row.get("confidence_diff")?,
            entry_diff: row.get("entry_diff")?,
            stop_diff: row.get("stop_diff")?,
            target_diff: row.get("target_diff")?,
            pattern_comparisons: vec![], // Would need separate table
            regime_comparison: crate::parity::RegimeComparison {
                rust_regime: row.get("rust_regime")?,
                ts_regime: row.get("ts_regime")?,
                agreement: row.get::<_, String>("rust_regime")?
                    == row.get::<_, String>("ts_regime")?,
            },
            agreement_score: row.get("agreement_score")?,
            notes: row.get::<_, Option<String>>("notes")?.unwrap_or_default(),
        })
    }
}

/// Filter for querying comparisons
pub struct ComparisonFilter {
    pub symbol: Option<String>,
    pub timeframe: Option<String>,
    pub comparison_type: Option<ComparisonType>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl Default for ComparisonFilter {
    fn default() -> Self {
        Self {
            symbol: None,
            timeframe: None,
            comparison_type: None,
            from: None,
            to: None,
        }
    }
}

/// Statistics stored in database
#[derive(Debug, Clone)]
pub struct StoredStatistics {
    pub total_comparisons: u64,
    pub exact_matches: u64,
    pub close_matches: u64,
    pub partial_matches: u64,
    pub disagreements: u64,
    pub misses: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
    pub direction_agreement_pct: f64,
    pub avg_confidence_diff: f64,
    pub avg_agreement_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_in_memory() {
        let storage = Storage::new_in_memory().unwrap();
        // Should succeed without panics
    }

    #[test]
    fn test_insert_and_retrieve() {
        let storage = Storage::new_in_memory().unwrap();

        let record = SignalComparisonRecord {
            comparison_id: "test-1".to_string(),
            timestamp: Utc::now(),
            symbol: "EURUSD".to_string(),
            timeframe: "M15".to_string(),
            ts_output: SignalOutput {
                timestamp: Utc::now(),
                symbol: "EURUSD".to_string(),
                timeframe: "M15".to_string(),
                direction: SignalDirection::Long,
                confidence: 75.0,
                confluence_score: 7.5,
                entry_price: Some(1.0850),
                stop_loss: Some(1.0820),
                take_profit: Some(1.0900),
                patterns: vec!["BOS".to_string()],
                regime: "TRENDING".to_string(),
                session: "LONDON".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            rust_output: SignalOutput {
                timestamp: Utc::now(),
                symbol: "EURUSD".to_string(),
                timeframe: "M15".to_string(),
                direction: SignalDirection::Long,
                confidence: 76.0,
                confluence_score: 7.5,
                entry_price: Some(1.0851),
                stop_loss: Some(1.0821),
                take_profit: Some(1.0901),
                patterns: vec!["BOS".to_string()],
                regime: "TRENDING".to_string(),
                session: "LONDON".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            comparison_type: ComparisonType::CloseMatch,
            direction_match: true,
            confidence_diff: 1.0,
            entry_diff: Some(0.0001),
            stop_diff: Some(0.0001),
            target_diff: Some(0.0001),
            pattern_comparisons: vec![],
            regime_comparison: crate::parity::RegimeComparison {
                rust_regime: "TRENDING".to_string(),
                ts_regime: "TRENDING".to_string(),
                agreement: true,
            },
            agreement_score: 0.95,
            notes: "Test comparison".to_string(),
        };

        storage.insert_comparison(&record).unwrap();

        let filter = ComparisonFilter::default();
        let results = storage.get_comparisons(filter, 10).unwrap();
        assert_eq!(results.len(), 1);
    }
}

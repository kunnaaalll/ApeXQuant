use crate::validation::certification::CertificationResult;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use serde_json;

pub struct CertificationReporter;

impl CertificationReporter {
    pub fn generate_markdown_report(result: &CertificationResult, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "# Performance Engine Certification Report")?;
        writeln!(file, "")?;
        writeln!(file, "## Status")?;
        writeln!(file, "**Current Certification State:** {:?}", result.state)?;
        writeln!(file, "")?;
        writeln!(file, "## Validation Criteria")?;
        writeln!(file, "- **Parity Achieved**: {}", if result.passes_parity { "✅" } else { "❌" })?;
        writeln!(file, "- **Determinism Verified**: {}", if result.passes_determinism { "✅" } else { "❌" })?;
        writeln!(file, "- **Benchmark Passed**: {}", if result.passes_benchmark { "✅" } else { "❌" })?;
        writeln!(file, "- **Replay Verified**: {}", if result.passes_replay { "✅" } else { "❌" })?;
        writeln!(file, "- **Monte Carlo Tolerated**: {}", if result.passes_monte_carlo { "✅" } else { "❌" })?;
        writeln!(file, "- **Stress Tests Passed**: {}", if result.passes_stress_tests { "✅" } else { "❌" })?;

        Ok(())
    }

    pub fn generate_json_report(result: &CertificationResult, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, result)?;
        Ok(())
    }
}

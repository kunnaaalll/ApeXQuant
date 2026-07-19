use crate::shadow::statistics::ShadowStatistics;
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct ShadowReporter;

impl ShadowReporter {
    pub fn generate_markdown_report(stats: &ShadowStatistics, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "# Shadow Mode Parity Report")?;
        writeln!(file)?;
        writeln!(file, "## Overview")?;
        writeln!(file, "- **Total Evaluations**: {}", stats.total_evaluations)?;
        writeln!(
            file,
            "- **Overall Agreement**: {}%",
            stats.get_overall_agreement_percentage()
        )?;
        writeln!(file, "- **Average Drift**: {}", stats.get_average_drift())?;
        writeln!(file, "- **Maximum Drift**: {}", stats.max_drift)?;
        writeln!(file)?;
        writeln!(file, "## Breakdown")?;
        writeln!(file, "- **Exact Matches**: {}", stats.exact_matches)?;
        writeln!(file, "- **Close Matches**: {}", stats.close_matches)?;
        writeln!(file, "- **Warnings**: {}", stats.warnings)?;
        writeln!(file, "- **Mismatches**: {}", stats.mismatches)?;
        writeln!(file, "- **Critical Failures**: {}", stats.critical_failures)?;
        writeln!(file)?;
        writeln!(file, "## Conclusion")?;
        if stats.get_overall_agreement_percentage() >= rust_decimal::Decimal::new(99, 0) {
            writeln!(file, "**Status:** PARITY ACHIEVED")?;
        } else {
            writeln!(file, "**Status:** PARITY NOT ACHIEVED")?;
        }

        Ok(())
    }

    pub fn generate_json_report(stats: &ShadowStatistics, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, stats)?;
        Ok(())
    }
}

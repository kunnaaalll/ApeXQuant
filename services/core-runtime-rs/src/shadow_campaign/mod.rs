use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignState {
    Preparing,
    Running,
    Paused,
    Investigating,
    Certified,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowCampaignManager {
    pub state: CampaignState,
    pub days_run: u32,
    pub total_days_target: u32,
    pub trades_executed: u64,
    pub target_trades: u64,
    pub replay_mismatches: u64,
    pub silent_drift_detected: u64,
    pub duplicate_orders_detected: u64,
    pub parity_percentage: f64,
    pub recovery_success_rate: f64,
}

impl Default for ShadowCampaignManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowCampaignManager {
    pub fn new() -> Self {
        Self {
            state: CampaignState::Preparing,
            days_run: 0,
            total_days_target: 30,
            trades_executed: 0,
            target_trades: 100_000,
            replay_mismatches: 0,
            silent_drift_detected: 0,
            duplicate_orders_detected: 0,
            parity_percentage: 100.0,
            recovery_success_rate: 100.0,
        }
    }

    pub fn start(&mut self) {
        if self.state == CampaignState::Preparing {
            self.state = CampaignState::Running;
        }
    }

    pub fn record_trade(&mut self) {
        self.trades_executed += 1;
    }

    pub fn record_anomaly(
        &mut self,
        mismatch: bool,
        drift: bool,
        duplicate: bool,
        parity: Option<f64>,
        recovery: Option<f64>,
    ) {
        if mismatch {
            self.replay_mismatches += 1;
        }
        if drift {
            self.silent_drift_detected += 1;
        }
        if duplicate {
            self.duplicate_orders_detected += 1;
        }
        if let Some(p) = parity {
            self.parity_percentage = p;
        }
        if let Some(r) = recovery {
            self.recovery_success_rate = r;
        }
        self.check_auto_fail();
    }

    fn check_auto_fail(&mut self) {
        if self.parity_percentage < 99.99
            || self.replay_mismatches > 0
            || self.silent_drift_detected > 0
            || self.duplicate_orders_detected > 0
        {
            self.state = CampaignState::Failed;
        }
    }

    pub fn run_daily_certification(&mut self) {
        if self.state == CampaignState::Failed {
            return;
        }
        self.days_run += 1;
        self.check_auto_fail();

        if self.state != CampaignState::Failed && self.days_run >= self.total_days_target && self.trades_executed >= self.target_trades {
            self.state = CampaignState::Certified;
        }
        self.generate_daily_report();
    }

    pub fn generate_daily_report(&self) {
        let content = format!(
            "# Daily Shadow Report (Day {})\n\
            State: {:?}\n\
            Trades Executed: {}\n\
            Parity: {:.4}%\n\
            Replay Mismatches: {}\n\
            Silent Drift: {}\n\
            Duplicate Orders: {}\n\
            Recovery Success: {:.2}%\n",
            self.days_run,
            self.state,
            self.trades_executed,
            self.parity_percentage,
            self.replay_mismatches,
            self.silent_drift_detected,
            self.duplicate_orders_detected,
            self.recovery_success_rate
        );
        let _ = self.write_to_disk("DAILY_SHADOW_REPORT.md", &content, false);
    }

    pub fn generate_weekly_report(&self) {
        let content = format!(
            "# Weekly Certification Report (Day {})\n\
            State: {:?}\n\
            Trades Executed: {}\n\
            Parity: {:.4}%\n",
            self.days_run, self.state, self.trades_executed, self.parity_percentage
        );
        let _ = self.write_to_disk("WEEKLY_CERTIFICATION_REPORT.md", &content, false);
    }

    pub fn generate_final_report(&self) {
        let cert_status = if self.state == CampaignState::Certified {
            "Passed"
        } else {
            "Failed"
        };
        let content = format!(
            "# Final 30-Day Certification Report\n\
            Institutional Certification: {}\n\
            \n\
            ## Metrics\n\
            Total Days Run: {}\n\
            Total Trades Executed: {}\n\
            Final State: {:?}\n\
            Parity: {:.4}%\n\
            Replay Mismatches: {}\n\
            Silent Drift: {}\n\
            Duplicate Orders: {}\n\
            Recovery Success: {:.2}%\n",
            cert_status,
            self.days_run,
            self.trades_executed,
            self.state,
            self.parity_percentage,
            self.replay_mismatches,
            self.silent_drift_detected,
            self.duplicate_orders_detected,
            self.recovery_success_rate
        );
        let _ = self.write_to_disk("FINAL_30_DAY_CERTIFICATION_REPORT.md", &content, false);
    }

    fn write_to_disk(&self, filename: &str, content: &str, append: bool) -> std::io::Result<()> {
        // Path is relative to the current working directory, which in testing will be where cargo test runs.
        // For production, we would want a configurable path. We will write to the root of the project or current directory.
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(filename)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

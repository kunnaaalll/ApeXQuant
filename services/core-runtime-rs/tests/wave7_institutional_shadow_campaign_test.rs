use core_runtime_rs::{CampaignState, ShadowCampaignManager};

#[test]
fn test_wave7_institutional_shadow_campaign_simulation() {
    let mut manager = ShadowCampaignManager::new();
    manager.start();

    assert_eq!(manager.state, CampaignState::Running);

    let trades_per_day = 100_000 / 30 + 1; // around 3334 trades per day

    for day in 1..=30 {
        // Simulate trades for the day
        for _ in 0..trades_per_day {
            manager.record_trade();
        }

        // Simulate zero mismatches and 100% recovery for the day
        manager.record_anomaly(false, false, false, Some(100.0), Some(100.0));

        // Run daily certification
        manager.run_daily_certification();

        if day < 30 {
            assert_eq!(manager.state, CampaignState::Running);
        } else {
            assert_eq!(manager.state, CampaignState::Certified);
        }
    }

    assert!(manager.trades_executed >= 100_000);
    assert_eq!(manager.replay_mismatches, 0);
    assert_eq!(manager.silent_drift_detected, 0);
    assert_eq!(manager.duplicate_orders_detected, 0);
    assert_eq!(manager.parity_percentage, 100.0);
    assert_eq!(manager.recovery_success_rate, 100.0);

    // Final report is generated
    manager.generate_final_report();
}

#[test]
fn test_wave7_auto_fail_on_parity_drop() {
    let mut manager = ShadowCampaignManager::new();
    manager.start();

    manager.record_trade();
    manager.record_anomaly(false, false, false, Some(99.98), Some(100.0));

    assert_eq!(manager.state, CampaignState::Failed);
}

#[test]
fn test_wave7_auto_fail_on_mismatch() {
    let mut manager = ShadowCampaignManager::new();
    manager.start();

    manager.record_trade();
    manager.record_anomaly(true, false, false, Some(100.0), Some(100.0));

    assert_eq!(manager.state, CampaignState::Failed);
}

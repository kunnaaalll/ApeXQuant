#![allow(warnings, clippy::all, deprecated)]
use portfolio_engine::portfolio::events::PortfolioEvent;
use portfolio_engine::portfolio::registry::PortfolioRegistry;
use portfolio_engine::portfolio::snapshot::SnapshotFrequency;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

#[test]
fn test_registry_determinism() {
    let registry = PortfolioRegistry::new();
    let pos_id = Uuid::new_v4();

    let events = vec![
        PortfolioEvent::Deposit {
            amount: Decimal::from(10000),
        },
        PortfolioEvent::PositionOpened {
            position_id: pos_id,
            margin_used: Decimal::from(1000),
            exposure: Decimal::from(10000),
        },
        PortfolioEvent::PnlUpdate {
            position_id: pos_id,
            pnl_delta: Decimal::from(500),
        },
        PortfolioEvent::PositionClosed {
            position_id: pos_id,
            realized_pnl: Decimal::from(500),
            margin_released: Decimal::from(1000),
            exposure_released: Decimal::from(10000),
        },
        // Reset floating PnL associated with closed position
        PortfolioEvent::PnlUpdate {
            position_id: pos_id,
            pnl_delta: Decimal::from(-500),
        },
    ];

    for event in events {
        registry.dispatch(event).unwrap();
    }

    let final_state = registry.get_state().unwrap();
    assert_eq!(final_state.balance, Decimal::from(10500));
    assert_eq!(final_state.equity, Decimal::from(10500));
    assert_eq!(final_state.active_positions, 0);
    assert_eq!(final_state.used_margin, Decimal::ZERO);

    // Verify snapshots were created
    let realtime_snapshots = registry.get_snapshots(SnapshotFrequency::Realtime);
    assert_eq!(realtime_snapshots.len(), 5);
}

#[test]
fn test_registry_concurrency() {
    let registry = Arc::new(PortfolioRegistry::new());

    // Setup initial balance to avoid failing margin invariant immediately
    // if a position is opened before deposit
    registry
        .dispatch(PortfolioEvent::Deposit {
            amount: Decimal::from(1_000_000),
        })
        .unwrap();

    let mut handles = vec![];

    for _ in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let pos_id = Uuid::new_v4();

                registry_clone
                    .dispatch(PortfolioEvent::PositionOpened {
                        position_id: pos_id,
                        margin_used: Decimal::from(10),
                        exposure: Decimal::from(100),
                    })
                    .unwrap();

                registry_clone
                    .dispatch(PortfolioEvent::PnlUpdate {
                        position_id: pos_id,
                        pnl_delta: Decimal::from(5),
                    })
                    .unwrap();

                registry_clone
                    .dispatch(PortfolioEvent::PositionClosed {
                        position_id: pos_id,
                        realized_pnl: Decimal::from(5),
                        margin_released: Decimal::from(10),
                        exposure_released: Decimal::from(100),
                    })
                    .unwrap();

                registry_clone
                    .dispatch(PortfolioEvent::PnlUpdate {
                        position_id: pos_id,
                        pnl_delta: Decimal::from(-5),
                    })
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_state = registry.get_state().unwrap();

    // Initial balance (1M) + (10 threads * 100 loops * 5 realized PnL) = 1,000,000 + 5000 = 1,005,000
    assert_eq!(final_state.balance, Decimal::from(1_005_000));
    assert_eq!(final_state.active_positions, 0);
    assert_eq!(final_state.used_margin, Decimal::ZERO);
    assert_eq!(final_state.floating_pnl, Decimal::ZERO);
    assert_eq!(final_state.equity, Decimal::from(1_005_000));

    // 1 deposit + (10 threads * 100 loops * 4 events) = 1 + 4000 = 4001 snapshots
    assert_eq!(registry.get_version(), 4001);
    let realtime_snapshots = registry.get_snapshots(SnapshotFrequency::Realtime);
    assert_eq!(realtime_snapshots.len(), 4001);
}

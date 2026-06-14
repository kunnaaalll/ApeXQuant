use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityCurvePoint {
    pub timestamp: time::OffsetDateTime,
    pub balance: Decimal,
    pub equity: Decimal,
    pub pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityCurveSnapshot {
    pub version: u64,
    pub timestamp: time::OffsetDateTime,
    pub points: Vec<EquityCurvePoint>,
    pub high_water_mark: Decimal,
    pub low_water_mark: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquityCurve {
    pub points: Vec<EquityCurvePoint>,
    pub high_water_mark: Decimal,
    pub low_water_mark: Decimal,
    pub current_version: u64,
}

impl EquityCurve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            high_water_mark: Decimal::ZERO,
            low_water_mark: Decimal::ZERO,
            current_version: 0,
        }
    }

    pub fn record_point(&mut self, timestamp: time::OffsetDateTime, balance: Decimal, equity: Decimal, pnl: Decimal) {
        self.points.push(EquityCurvePoint {
            timestamp,
            balance,
            equity,
            pnl,
        });

        if self.points.len() == 1 || equity > self.high_water_mark {
            self.high_water_mark = equity;
        }
        if self.points.len() == 1 || equity < self.low_water_mark {
            self.low_water_mark = equity;
        }
        self.current_version += 1;
    }

    pub fn take_snapshot(&self, timestamp: time::OffsetDateTime) -> EquityCurveSnapshot {
        EquityCurveSnapshot {
            version: self.current_version,
            timestamp,
            points: self.points.clone(),
            high_water_mark: self.high_water_mark,
            low_water_mark: self.low_water_mark,
        }
    }

    pub fn replay_from_snapshot(&mut self, snapshot: &EquityCurveSnapshot) {
        self.points = snapshot.points.clone();
        self.high_water_mark = snapshot.high_water_mark;
        self.low_water_mark = snapshot.low_water_mark;
        self.current_version = snapshot.version;
    }
}

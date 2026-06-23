use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnomalySeverity {
    Minor,
    Warning,
    Major,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    SequenceJump(u64),
    PriceSpike(Decimal),
    VolumeSpike(Decimal),
    TimestampGap(u64),
    SpreadExplosion(Decimal),
}

pub struct Anomaly {
    pub kind: AnomalyType,
    pub severity: AnomalySeverity,
}

pub struct AnomalyEngine;

impl AnomalyEngine {
    pub fn detect_spread_explosion(spread: Decimal, baseline_spread: Decimal) -> Result<Option<Anomaly>, &'static str> {
        if spread < Decimal::ZERO || baseline_spread < Decimal::ZERO {
            return Err("Spread cannot be negative");
        }
        if baseline_spread.is_zero() {
            return Ok(None);
        }

        let ratio = spread / baseline_spread;
        let severity = if ratio > Decimal::from(10) {
            Some(AnomalySeverity::Critical)
        } else if ratio > Decimal::from(5) {
            Some(AnomalySeverity::Major)
        } else if ratio > Decimal::from(3) {
            Some(AnomalySeverity::Warning)
        } else if ratio > Decimal::from(2) {
            Some(AnomalySeverity::Minor)
        } else {
            None
        };

        Ok(severity.map(|sev| Anomaly {
            kind: AnomalyType::SpreadExplosion(spread),
            severity: sev,
        }))
    }
}

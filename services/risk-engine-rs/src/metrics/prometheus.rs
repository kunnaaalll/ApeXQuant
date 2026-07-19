use once_cell::sync::Lazy;
use prometheus::{opts, register_gauge, Gauge};
use tracing::error;

pub static RISK_CONFIDENCE: Lazy<Gauge> = Lazy::new(|| {
    match register_gauge!(opts!(
        "risk_confidence_score",
        "Current risk confidence score of the system"
    )) {
        Ok(g) => g,
        Err(e) => {
            error!("Failed to register risk_confidence_score: {}", e);
            Gauge::new("risk_confidence_score_dummy", "Dummy")
                .unwrap_or_else(|_| std::process::exit(1))
        }
    }
});

pub static RISK_DRAWDOWN: Lazy<Gauge> = Lazy::new(|| {
    match register_gauge!(opts!(
        "risk_drawdown_percent",
        "Current drawdown percentage of the system"
    )) {
        Ok(g) => g,
        Err(e) => {
            error!("Failed to register risk_drawdown_percent: {}", e);
            Gauge::new("risk_drawdown_percent_dummy", "Dummy")
                .unwrap_or_else(|_| std::process::exit(1))
        }
    }
});

pub static RISK_EXPOSURE: Lazy<Gauge> = Lazy::new(|| {
    match register_gauge!(opts!(
        "risk_total_exposure",
        "Current total exposure in USD"
    )) {
        Ok(g) => g,
        Err(e) => {
            error!("Failed to register risk_total_exposure: {}", e);
            Gauge::new("risk_total_exposure_dummy", "Dummy")
                .unwrap_or_else(|_| std::process::exit(1))
        }
    }
});

pub static VALUE_AT_RISK: Lazy<Gauge> = Lazy::new(|| {
    match register_gauge!(opts!(
        "risk_value_at_risk",
        "Current 99% Value at Risk in USD"
    )) {
        Ok(g) => g,
        Err(e) => {
            error!("Failed to register risk_value_at_risk: {}", e);
            Gauge::new("risk_value_at_risk_dummy", "Dummy")
                .unwrap_or_else(|_| std::process::exit(1))
        }
    }
});

pub fn record_metrics(confidence: f64, drawdown: f64, exposure: f64, var: f64) {
    RISK_CONFIDENCE.set(confidence);
    RISK_DRAWDOWN.set(drawdown);
    RISK_EXPOSURE.set(exposure);
    VALUE_AT_RISK.set(var);
}

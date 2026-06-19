#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub category: String,
    pub description: String,
    pub severity: u32,
}

pub fn track_reasons(
    drawdown: &str,
    exposure: &str,
    correlation: &str,
    tail_risk: &str,
    var: &str,
    circuit_breaker: &str,
) -> Vec<Reason> {
    let mut reasons = vec![
        Reason {
            category: "Drawdown".to_string(),
            description: drawdown.to_string(),
            severity: map_severity(drawdown),
        },
        Reason {
            category: "Exposure".to_string(),
            description: exposure.to_string(),
            severity: map_severity(exposure),
        },
        Reason {
            category: "Correlation".to_string(),
            description: correlation.to_string(),
            severity: map_severity(correlation),
        },
        Reason {
            category: "TailRisk".to_string(),
            description: tail_risk.to_string(),
            severity: map_severity(tail_risk),
        },
        Reason {
            category: "VaR".to_string(),
            description: var.to_string(),
            severity: map_severity(var),
        },
        Reason {
            category: "CircuitBreaker".to_string(),
            description: circuit_breaker.to_string(),
            severity: map_severity(circuit_breaker),
        },
    ];

    // Sort by severity descending
    reasons.sort_by_key(|b| std::cmp::Reverse(b.severity));
    reasons
}

fn map_severity(desc: &str) -> u32 {
    match desc {
        "Frozen" | "Collapse" | "Critical" => 100,
        "High" | "Warning" => 50,
        "Healthy" | "Safe" => 0,
        _ => 10, // Default base severity
    }
}

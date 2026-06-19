#[allow(clippy::too_many_arguments)]
pub fn detect_deterioration(
    prev_drawdown: u32,
    curr_drawdown: u32,
    prev_leverage: u32,
    curr_leverage: u32,
    prev_var: u32,
    curr_var: u32,
    correlation_spikes: bool,
    tail_risk_expansion: bool,
) -> Vec<String> {
    let mut deterioration = Vec::new();

    if curr_drawdown > prev_drawdown {
        deterioration.push("rising drawdown".to_string());
    }

    if curr_leverage > prev_leverage {
        deterioration.push("rising leverage".to_string());
    }

    if curr_var > prev_var {
        deterioration.push("rising VaR".to_string());
    }

    if correlation_spikes {
        deterioration.push("correlation spikes".to_string());
    }

    if tail_risk_expansion {
        deterioration.push("tail-risk expansion".to_string());
    }

    deterioration
}

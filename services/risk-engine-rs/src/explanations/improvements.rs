pub fn detect_improvements(
    prev_drawdown: u32,
    curr_drawdown: u32,
    prev_exposure: u32,
    curr_exposure: u32,
    prev_var: u32,
    curr_var: u32,
    in_cooldown_recovery: bool,
) -> Vec<String> {
    let mut improvements = Vec::new();

    if curr_drawdown < prev_drawdown {
        improvements.push("decreasing drawdown".to_string());
    }

    if curr_exposure < prev_exposure {
        improvements.push("lower exposure".to_string());
    }

    if curr_var < prev_var {
        improvements.push("reduced VaR".to_string());
    }

    if in_cooldown_recovery {
        improvements.push("cooldown recovery".to_string());
    }

    improvements
}

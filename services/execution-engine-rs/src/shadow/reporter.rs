use super::state::ShadowState;

pub struct ReporterEngine;

impl ReporterEngine {
    pub fn generate_markdown(state: &ShadowState) -> String {
        let stats = &state.statistics;
        
        let drift_str = match &state.drift_score {
            Some(d) => format!("- Absolute: {}\n- Relative: {}%\n- State: {:?}", d.absolute_drift, d.relative_drift, d.state),
            None => "- Not recorded".to_string(),
        };

        let parity_str = match &state.parity_score {
            Some(p) => format!("- Score: {}\n- Level: {:?}", p.value, p.level),
            None => "- Not recorded".to_string(),
        };

        let health_str = match &state.health {
            Some(h) => format!("{:?}", h),
            None => "Unknown".to_string(),
        };

        format!(
            "# Shadow Mode Report\n\n\
            ## Statistics\n\
            - Daily Match Rate: {}%\n\
            - Weekly Match Rate: {}%\n\
            - Monthly Match Rate: {}%\n\
            - Exact Matches: {}\n\
            - Close Matches: {}\n\
            - Warnings: {}\n\
            - Mismatches: {}\n\
            - Critical Mismatches: {}\n\n\
            ## Drift\n\
            {}\n\n\
            ## Parity\n\
            {}\n\n\
            ## Health\n\
            - Status: {}\n\n\
            ## Validator Status\n\
            - State: {:?}\n\
            - Streaks: {}\n",
            stats.daily_match_rate,
            stats.weekly_match_rate,
            stats.monthly_match_rate,
            stats.exact_match_count,
            stats.close_match_count,
            stats.warning_count,
            stats.mismatch_count,
            stats.critical_mismatch_count,
            drift_str,
            parity_str,
            health_str,
            state.validator.state,
            state.validator.consecutive_parity_streaks
        )
    }

    pub fn generate_json(state: &ShadowState) -> String {
        let stats = &state.statistics;
        
        let drift_abs = state.drift_score.as_ref().map_or("0".to_string(), |d| d.absolute_drift.to_string());
        let drift_rel = state.drift_score.as_ref().map_or("0".to_string(), |d| d.relative_drift.to_string());
        
        let parity_val = state.parity_score.as_ref().map_or("0".to_string(), |p| p.value.to_string());

        let health_val = match &state.health {
            Some(h) => format!("{:?}", h),
            None => "Unknown".to_string(),
        };

        format!(
            "{{\n\
            \t\"statistics\": {{\n\
            \t\t\"daily_match_rate\": \"{}\",\n\
            \t\t\"exact_matches\": {},\n\
            \t\t\"critical_mismatches\": {}\n\
            \t}},\n\
            \t\"drift\": {{\n\
            \t\t\"absolute\": \"{}\",\n\
            \t\t\"relative\": \"{}\"\n\
            \t}},\n\
            \t\"parity\": {{\n\
            \t\t\"score\": \"{}\"\n\
            \t}},\n\
            \t\"health\": \"{}\",\n\
            \t\"validator\": {{\n\
            \t\t\"state\": \"{:?}\",\n\
            \t\t\"streaks\": {}\n\
            \t}}\n\
            }}",
            stats.daily_match_rate,
            stats.exact_match_count,
            stats.critical_mismatch_count,
            drift_abs,
            drift_rel,
            parity_val,
            health_val,
            state.validator.state,
            state.validator.consecutive_parity_streaks
        )
    }
}

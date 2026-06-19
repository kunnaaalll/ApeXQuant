use crate::shadow::statistics::StatisticsEngine;

#[derive(Debug, Clone)]
pub struct Reporter;

impl Default for Reporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_markdown_report(&self, stats: &StatisticsEngine) -> String {
        format!(
            "# Shadow Mode Report\n\n\
            ## Daily Statistics\n\
            - Exact Matches: {}\n\
            - Close Matches: {}\n\
            - Warnings: {}\n\
            - Mismatches: {}\n\n\
            **Match Percentage**: {}%\n",
            stats.daily_exact_matches,
            stats.daily_close_matches,
            stats.daily_warnings,
            stats.daily_mismatches,
            stats.match_percentage()
        )
    }

    pub fn generate_json_report(&self, stats: &StatisticsEngine) -> String {
        format!(
            "{{\n\
            \x20\x20\"daily\": {{\n\
            \x20\x20\x20\x20\"exact_matches\": {},\n\
            \x20\x20\x20\x20\"close_matches\": {},\n\
            \x20\x20\x20\x20\"warnings\": {},\n\
            \x20\x20\x20\x20\"mismatches\": {}\n\
            \x20\x20}},\n\
            \x20\x20\"match_percentage\": \"{}\"\n\
            }}",
            stats.daily_exact_matches,
            stats.daily_close_matches,
            stats.daily_warnings,
            stats.daily_mismatches,
            stats.match_percentage()
        )
    }
}

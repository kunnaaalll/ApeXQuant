#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationHealth {
    Excellent,
    Good,
    Normal,
    Weak,
    Critical,
}

impl ValidationHealth {
    pub fn derive(
        parity_pass: bool,
        determinism_pass: bool,
        replay_pass: bool,
        stress_pass: bool,
        benchmark_pass: bool,
    ) -> Self {
        let mut score = 0;
        if parity_pass { score += 1; }
        if determinism_pass { score += 1; }
        if replay_pass { score += 1; }
        if stress_pass { score += 1; }
        if benchmark_pass { score += 1; }

        match score {
            5 => ValidationHealth::Excellent,
            4 => ValidationHealth::Good,
            3 => ValidationHealth::Normal,
            2 => ValidationHealth::Weak,
            _ => ValidationHealth::Critical,
        }
    }
}

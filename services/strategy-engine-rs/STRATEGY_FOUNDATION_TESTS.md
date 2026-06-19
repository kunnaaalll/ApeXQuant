# Strategy Foundation Tests

Test execution covers the following domains:
1. `test_state_transitions`: Validates illegal transitions.
2. `test_score_bounds`: Ensures boundaries clamping behavior.
3. `test_degradation_thresholds`: Validates the mapping of decays to early warnings or collapse.
4. `test_recovery_logic`: Ensures multi-cycle stability requirements.
5. `test_determinism`: Iterates operations 100,000 times checking for floating point imprecision drift.

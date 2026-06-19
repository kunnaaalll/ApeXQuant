# STRESS ENGINE TESTS

## Testing Philosophy

Tests are written considering the highest degree of safety and reliability standards required by the APEX V3 specifications.

## Core Validations

1. **Survival Score Bounds**
   Validates that the survival score computed strictly rests between 0 and 100 without exceptions.

2. **Determinism (100k iterations)**
   Executes calculations inside a loop verifying that intermediate precision representations do not cause numeric drift.

3. **Correlation Convergence**
   Ensures that correlation collapse functions always trend toward, but do not exceed, the maximum bound of `1.0`.

4. **Leverage Cascade**
   Asserts that extreme amplifications transition states cleanly without unexpected overflow.

5. **Event Rebuild**
   Verifies that replayable structs (`StressSnapshot`) are serialized and deserialized flawlessly to reconstruct past events.

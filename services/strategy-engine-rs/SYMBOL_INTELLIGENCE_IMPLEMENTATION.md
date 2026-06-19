# Symbol Intelligence Implementation

## Overview
The Symbol Intelligence module calculates instrument-specific strategy viability while severely penalizing low sample counts.

## Sample Count Penalties
- < 100 samples applies a mild penalty
- < 50 samples applies a severe 50% cut
- < 20 samples applies a critical 90% cut

## Output
Generates deterministic grades (`Elite` through `Forbidden`) to prevent deployment on untested assets.

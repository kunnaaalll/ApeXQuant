# Correlation Engine

The Correlation engine calculates rolling Pearson correlation coefficients between dual assets or synthetic returns, entirely using bounded integer math without floats.

## Details

- Stores standard return histories for sets A and B.
- Computes covariance and standard deviation dynamically on `.update()`.
- Constrains correlation scores exactly between `-100` and `+100` mapping to `[-1.0, 1.0]`.
- Enforces strict zero-handling guarantees to prevent divide-by-zero panics.

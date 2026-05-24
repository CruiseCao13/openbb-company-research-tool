# Codex Spot Check Report

## Sampling Rule

This local spot check samples all hard failures plus a deterministic cross-section of generated reports. It is a guard against trusting summary scores blindly.

## Sampled Reports

| Ticker | Why Sampled | Score | Grade | Observed Issues |
|---|---|---:|---|---|
| AAPL | profile/coverage sample | 86 | GOOD | None |
| GOOGL | profile/coverage sample | 86 | GOOD | None |
| META | profile/coverage sample | 86 | GOOD | None |
| MSFT | profile/coverage sample | 86 | GOOD | None |
| NVDA | profile/coverage sample | 86 | GOOD | None |

## Judgment

The run can be accepted only when hard failures are understood and training cases are generated for weak outputs. High scores still require periodic manual review before broad_500 is treated as product-grade.

# Codex Spot Check Report

## Sampling Rule

This local spot check samples all hard failures plus a deterministic cross-section of generated reports. It is a guard against trusting summary scores blindly.

## Sampled Reports

| Ticker | Why Sampled | Score | Grade | Observed Issues |
|---|---|---:|---|---|
| ACHR | profile/coverage sample | 78 | ACCEPTABLE | None |
| ASTS | profile/coverage sample | 86 | GOOD | None |
| GSL | profile/coverage sample | 86 | GOOD | None |
| JBHT | profile/coverage sample | 86 | GOOD | None |
| JOBY | profile/coverage sample | 86 | GOOD | None |
| MATX | profile/coverage sample | 86 | GOOD | None |
| ODFL | profile/coverage sample | 86 | GOOD | None |
| PLTR | profile/coverage sample | 86 | GOOD | None |
| RKLB | profile/coverage sample | 78 | ACCEPTABLE | None |
| SOFI | profile/coverage sample | 86 | GOOD | None |

## Judgment

The run can be accepted only when hard failures are understood and training cases are generated for weak outputs. High scores still require periodic manual review before broad_500 is treated as product-grade.

# Regression Hard Cases Report

Generated: 2026-05-25

## Eval Set

Path: `eval_sets/regression_hard_cases.yaml`

Groups: 13
Tickers: 18
Unique tickers: 18

Required cases present:

- LUNR
- RKLB
- AAPL
- GOOGL
- META
- CAT
- ISRG
- LLY
- AMD
- NVDA
- T
- ASTS
- ZIM
- INTC
- JPM
- 600519.SH
- 000001.SZ
- 300750.SZ

## Local Regression Run

Command:

```bash
cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- batch eval_sets/regression_hard_cases.yaml \
  --workers 2 \
  --ai local \
  --run-id ci_regression_hard_cases_local \
  --force
```

Output: `reports/batch_runs/ci_regression_hard_cases_local`

Result:

- Total tickers: 18
- PASS: 0
- WARNING: 18
- FAIL: 0
- External AI calls: 0
- Local fallback reports: 18

Final status: PASS for local regression smoke. External quality validation remains separate.

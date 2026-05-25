# broad_500 Dry Run Report

Generated: 2026-05-25

## Eval Set

Path: `eval_sets/broad_500_us_cn.yaml`

Groups: 34
Tickers: 500
Unique tickers: 500

Required hard cases are present, including LUNR, RKLB, AAPL, GOOGL, CAT, ISRG, LLY, AMD, NVDA, T, ASTS, ZIM, INTC, JPM, 600519.SH, 000001.SZ, and 300750.SZ.

## Dry Run

Command:

```bash
cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- train eval_sets/broad_500_us_cn.yaml \
  --stage broad_500 \
  --workers 2 \
  --ai local \
  --limit 10 \
  --offset 0 \
  --budget-calls 0 \
  --max-iterations 1 \
  --run-id ci_broad_500_dryrun_10 \
  --force
```

Output: `reports/training_runs/ci_broad_500_dryrun_10`

Artifacts:

- `quality_matrix.csv`
- `quality_matrix.json`
- `training_cases_generated.jsonl`
- `regression_cases_generated.jsonl`
- `cost_report.md`
- `model_improvement_review.md`

Result:

- Reports scored: 10
- Average quality score: 78.0
- Training cases generated: 10
- External calls: 0
- Local fallback reports: 10

Final status: PASS for local dry run. Not external training.

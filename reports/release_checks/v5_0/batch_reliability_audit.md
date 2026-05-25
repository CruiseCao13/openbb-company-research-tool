# Batch Reliability Audit

Generated: 2026-05-25 Asia/Singapore

Command run from `research-rs/`:

```bash
cargo run -p research-rs -- batch ../eval_sets/broad_30_probe.yaml --workers 2 --ai local --run-id v5_consistency_broad_30_audit
```

## Output

Path: `reports/batch_runs/v5_consistency_broad_30_audit`

| Metric | Value |
|---|---:|
| Total | 30 |
| PASS | 24 |
| WARNING | 6 |
| FAIL | 0 |
| External AI calls | 0 |
| Local fallback reports | 30 |
| Training cases generated | 6 |
| Avg runtime per ticker ms | 3281 |

Required files exist: batch summary, failures, warnings, profile distribution, credit usage, training cases, dashboard, and batch trace.

Status: PASS

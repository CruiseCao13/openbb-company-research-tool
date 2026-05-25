# Full Stabilization Autopilot Attempt Log

Generated: 2026-05-25

| Attempt | Failed check | Root cause | Files changed | Validation command | Result | Remaining blockers |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | CI workflow consistency | CI provider syntax step used `python`; local required command uses `python3` | `.github/workflows/ci.yml`, `.github/workflows/provider-smoke.yml` | local CI command sequence | PASS | none |
| 2 | Training case hygiene | Batch runner wrote all correction cases to polluted legacy `v5_correction_cases.jsonl`; eval parser could read non-ticker list items under rules | `research-batch/src/eval_set.rs`, `runner.rs`, `training_case.rs`, tests, split correction files | `cargo test`, regeneration batch | PASS | external correction cases remain empty until real external training run |
| 3 | Secret scan noise | Placeholder and release-check recursive matches were treated like hard secret hits | `research-cli/src/main.rs`, `secret_scan_policy.md` | secret scan tests and real-key rg scan | PASS | none |
| 4 | broad_500 dry-run proof | Needed current local dry-run artifact after training hygiene fix | training run artifacts under `reports/training_runs/ci_broad_500_dryrun_10` | `research-rs train ... --budget-calls 0` | PASS | not external training |

Consecutive no-progress attempts: 0

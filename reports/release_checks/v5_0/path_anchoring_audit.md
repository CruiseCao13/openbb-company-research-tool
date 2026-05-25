# Root Path Anchoring Audit

Generated: 2026-05-25 Asia/Singapore

## Runtime Check

`cargo run -p research-rs -- run AAPL --ai local --run-id path_audit_aapl --pack` completed from `research-rs/` and wrote to repo-root `reports/AAPL/runs/path_audit_aapl`.

`find research-rs/reports -type f` result: `(none)`

During this final audit, batch generation also exposed a stale cwd-rooted `research-rs/training_cases/corrections/v5_correction_cases.jsonl`. That was fixed by routing global v5 training case output through the shared repo-root resolver.

`research-rs/training_cases/` result after cleanup: `(absent)`

## Source Findings

| Generated Path Family | Resolver / Evidence | Status |
|---|---|---|
| reports root | `research_core::paths::reports_root()` | PASS |
| AI cache | `research_core::paths::ai_cache_dir()` | PASS |
| batch runs | `research_core::paths::batch_runs_dir()` | PASS |
| quality runs | `research_core::paths::quality_runs_dir()` | PASS |
| samples | `research_core::paths::samples_dir()` | PASS |
| release checks | `research_core::paths::release_checks_dir()` | PASS |
| global training cases | `research_core::paths::training_cases_dir()` | PASS |
| provider scripts | repo-root provider resolver | PASS |

References to `reports/samples/...` inside README/tests are assertions or sample links, not generated output roots.

## Remaining Risk

The audit now covers the path families that were observed during run, batch, cache, samples, release checks, and training case generation. Any future generated path family should be added to `research_core::paths` rather than constructed from cwd.

Status: PASS

# v5 Path Anchoring Audit

Generated: 2026-05-25 Asia/Singapore

## Scope

Searched Rust crates for generated-output path construction patterns:

```bash
rg -n "PathBuf::from\(\"reports\"|Path::new\(\"reports\"|current_dir\(\).*reports|reports/_cache|_cache/ai|batch_runs|quality_runs|samples|release_checks" research-rs/crates
```

## Findings

| Path Family | Result | Evidence |
|---|---|---|
| AI cache | PASS | `research_core::paths::ai_cache_dir()` resolves to repo-root `reports/_cache/ai`. |
| Batch runs | PASS | `research_core::paths::batch_runs_dir()` resolves to repo-root `reports/batch_runs`. |
| Quality runs | PASS | `research_core::paths::quality_runs_dir()` resolves to repo-root `reports/quality_runs`. |
| Samples | PASS | `research_core::paths::samples_dir()` resolves to repo-root `reports/samples`. |
| Release checks | PASS | `research_core::paths::release_checks_dir()` resolves to repo-root `reports/release_checks`. |
| Direct `PathBuf::from("reports")` | PASS | No generated-output construction found in crates. |
| Direct `Path::new("reports")` | PASS | No generated-output construction found in crates. |
| `current_dir().join("reports")` | PASS | No generated-output construction found in crates. |
| `research-rs/reports` | PASS | Directory absent; `.gitignore` remains defensive. |

References to `reports/samples/...` inside tests and README checks are assertions against expected repo-root files, not output roots.

## Result

Generated v5 output paths are anchored through the shared repo-root path resolver.

Status: PASS

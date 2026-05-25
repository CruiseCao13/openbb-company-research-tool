# v6 Tauri Studio Phase 12: Regression Matrix Hub Basic

Final status: PASS

## Files Changed

- `src-tauri/src/lib.rs`
- `studio/src/App.tsx`
- `studio/src/lib/tauri.ts`
- `studio/src/styles.css`
- `studio/src/types/app.ts`
- `reports/release_checks/v6_0/phase_12_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_12_regression_matrix_report.md`

## Commands Implemented

- `list_training_runs()`
  - Scans `reports/training_runs/*/`.
  - Returns run id, path, quality matrix presence, issue distribution presence, training case presence, generated timestamp when available, and summary status when available.
  - Missing `reports/training_runs` returns an empty list.

- `load_quality_matrix(run_id)`
  - Reads `reports/training_runs/{run_id}/quality_matrix.json`.
  - Falls back to `quality_matrix.csv`.
  - Reads `issue_distribution.json` when available; otherwise derives issue counts from row issue and hard-failure fields.
  - Rejects unsafe `run_id` path segments and keeps access under `repo_root/reports/training_runs`.
  - Does not mutate artifacts, run training, call providers, or call external AI.

## Matrix DTO Shape

`TrainingRunSummary`:

- `run_id`
- `path`
- `has_quality_matrix`
- `has_issue_distribution`
- `has_training_cases`
- `generated_at`
- `summary_status`

`QualityMatrix`:

- `run_id`
- `rows`
- `issue_distribution`
- `warnings`

`QualityMatrixRow`:

- `ticker`
- `market`
- `company_name`
- `status`
- `quality_score`
- `grade`
- `issue_types`
- `hard_failures`
- `ai_source`
- `provider_status`

## UI Behavior

- Added a top-level mode switch with `Research Runs` and `Regression Matrix`.
- Regression view loads training runs through Tauri IPC only.
- Training run selector loads existing matrix artifacts for the selected run.
- Summary cards show ticker count, average quality, warning count, hard-failure count, and provider-failure count.
- Matrix cells render a dense quality-control board.
- Selecting a ticker cell opens a side panel with score, grade, status, issue types, hard failures, AI source, and provider status.
- Empty, missing, malformed, failed, and browser-preview states are visible and non-crashing.

## Color / Status Mapping

- `quality_score >= 85`: green / pass treatment.
- `70 <= quality_score < 85`: blue-info / acceptable-good treatment.
- `60 <= quality_score < 70`: orange / weak treatment.
- `quality_score < 60`: rose / fail treatment.
- Missing score: slate / unknown treatment.

## Empty / Malformed State Behavior

- Missing training runs folder: empty state.
- Missing quality matrix: empty rows plus warning.
- Missing CSV columns: available columns are parsed; absent fields become null.
- Malformed JSON: typed backend error surfaced in UI.
- Browser preview without Tauri IPC: explicit runtime warning, no fake discovery.

## Tests Added

Rust/Tauri tests:

- `list_training_runs_returns_empty_when_missing`
- `list_training_runs_detects_quality_matrix`
- `load_quality_matrix_reads_json`
- `load_quality_matrix_reads_csv_fallback`
- `load_quality_matrix_handles_missing_columns`
- `load_quality_matrix_rejects_path_traversal`
- `quality_matrix_path_stays_under_reports`
- `issue_distribution_parsed_when_available`

No frontend test framework was introduced in this phase.

## Validation Commands Run

- `npm run typecheck` - PASS
- `npm run build` - PASS
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` - PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` - PASS
- `cargo build --manifest-path src-tauri/Cargo.toml` - PASS
- `cargo test --manifest-path src-tauri/Cargo.toml` - PASS, 44 tests passed
- `cargo test --manifest-path research-rs/Cargo.toml` - PASS
- `git diff --check` - PASS

Note: `npm install` was run to restore local frontend dependencies for validation. Generated `node_modules`, `studio/dist`, and `src-tauri/target` artifacts were removed before staging.

## Intentionally Not Implemented

- No D3.
- No Sankey.
- No PDF export.
- No external training.
- No 500-company training run.
- No quality matrix generation.
- No training artifact mutation.
- No provider or external OpenAI calls.

## Unrelated Files Left Untouched

None observed in the preflight workspace.

## Remaining Blockers

None for Phase 12 basic Regression Matrix Hub.

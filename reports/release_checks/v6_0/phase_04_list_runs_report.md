# v6.0 Phase 04 list_runs IPC Report

Final status: PASS

## 1. Files Changed

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/lib.rs`
- `studio/src/App.tsx`
- `studio/src/lib/tauri.ts`
- `studio/src/styles.css`
- `studio/src/types/app.ts`
- `reports/release_checks/v6_0/phase_04_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_04_list_runs_report.md`

No v5 provider, AI prompt, validator, training loop, report generation, eval set, or generated run-folder logic was modified.

## 2. Command Implemented

Implemented Tauri IPC command:

```rust
list_runs() -> Result<Vec<RunSummary>, String>
```

Behavior:

- Rust resolves the repo root using the Phase 3 backend resolver.
- Rust validates `reports_root = repo_root/reports` remains under the repo root.
- Rust scans `reports/*/runs/*/`.
- Missing metadata is tolerated and returned as `null` fields.
- Partial run folders still appear in the list.
- No report markdown body is parsed.
- No provider, network, external AI, or mutation logic is invoked.

## 3. RunSummary DTO Shape

```json
{
  "ticker": "string",
  "run_id": "string",
  "market": "string | null",
  "provider": "string | null",
  "status": "string | null",
  "ai_source": "string | null",
  "external_ai_used": "boolean | null",
  "local_mock_used": "boolean | null",
  "cache_hits": "number | null",
  "new_external_ai_calls": "number | null",
  "human_review_required": "boolean | null",
  "generated_at": "string | null",
  "report_path_exists": "boolean",
  "dashboard_path_exists": "boolean",
  "pdf_path_exists": "boolean",
  "run_folder": "string"
}
```

Metadata read attempts:

- `metadata/report_status.json`
- `metadata/ai_usage.json`
- `raw/provider_payload.json`

Artifact existence checks:

- report markdown under `report/`
- `dashboard.html`
- report PDF under `report/`

## 4. Frontend List Behavior

The left sidebar now calls `list_runs()` on app start and renders real run summaries when running inside the Tauri runtime.

States implemented:

- `Loading runs...`
- empty state when no run folders are found
- error state when the command fails
- browser-preview warning when Tauri IPC is unavailable

Each run item displays:

- ticker
- run id
- status badge
- AI source badge
- human review marker when flagged
- market/provider small text

Clicking a run only selects it in the UI and shows a lightweight selected-run summary in the main panel. It does not load full report detail.

## 5. Error / Empty / Loading States

- Loading: shows `Loading runs...`
- Empty: shows `No runs found`
- Failed: shows the command error
- Browser preview: shows that real run discovery requires the Tauri runtime

The app does not crash when Tauri IPC is unavailable.

## 6. Tests Added

Rust/Tauri tests were added using temp fixture directories:

- `list_runs_returns_empty_when_reports_missing`
- `list_runs_reads_basic_report_status`
- `list_runs_handles_missing_ai_usage`
- `list_runs_detects_artifact_existence`
- `list_runs_does_not_crash_on_partial_run`
- `list_runs_sorts_stably`

Existing Phase 3 tests still pass:

- `get_app_info_returns_required_fields`
- `get_app_info_reports_root_points_to_repo_reports`

## 7. Validation Commands Run

Frontend:

```bash
npm install
npm run typecheck
npm run build
```

Tauri:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Existing v5:

```bash
cargo test --manifest-path research-rs/Cargo.toml
```

Git whitespace:

```bash
git diff --check
```

## 8. Validation Results

- `npm install`: PASS, with 2 moderate npm audit advisories reported by npm.
- `npm run typecheck`: PASS.
- `npm run build`: PASS.
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 8 tests passed.
- `cargo test --manifest-path research-rs/Cargo.toml`: PASS, 152 tests passed.
- `git diff --check`: PASS.

No external OpenAI API, provider network, training run, or real report content loading was used.

## 9. Intentionally Not Implemented

Phase 4 intentionally does not implement:

- `load_run_detail`;
- reading full report markdown;
- report rendering;
- D3;
- PDF;
- regression matrix;
- provider calls;
- external AI calls;
- mutation of run folders;
- v5 core analysis changes.

## 10. Unrelated Files Left Untouched

Left unstaged and untouched:

- `reports/release_checks/v5_0/a_share_sector_smoke_report.md`

This is unrelated to the v6 Phase 4 implementation.

## 11. Remaining Blockers

No blockers for Phase 4.

Known non-blocking note: npm reports 2 moderate dependency advisories. They were not force-fixed because that could introduce unrelated dependency churn.

## 12. Next Phase Recommendation

Phase 5 should add a typed `load_run_detail` contract that reads a narrow set of metadata artifacts only. Full markdown rendering, charts, PDF, and regression matrix should remain out of scope until the DTO boundary is stable.

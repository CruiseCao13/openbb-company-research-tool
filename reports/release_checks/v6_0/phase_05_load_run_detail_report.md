# v6.0 Phase 05 load_run_detail IPC Report

Final status: PASS

## 1. Files Changed

- `src-tauri/src/lib.rs`
- `studio/src/App.tsx`
- `studio/src/lib/tauri.ts`
- `studio/src/styles.css`
- `studio/src/types/app.ts`
- `reports/release_checks/v6_0/phase_05_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_05_load_run_detail_report.md`

No v5 provider, AI prompt, validator, training loop, report generation, financial calculation, eval set, or generated run-folder logic was modified.

## 2. Command Implemented

Implemented Tauri IPC command:

```rust
load_run_detail(ticker: String, run_id: String) -> Result<RunDetail, String>
```

Safety behavior:

- validates `ticker` and `run_id` as safe path segments;
- rejects path traversal;
- resolves `reports/{ticker}/runs/{run_id}/`;
- validates the resolved run folder stays under repo-root `reports`;
- returns a typed error if the run folder is missing;
- does not mutate run folders;
- does not call providers;
- does not call external AI;
- does not parse or return full report markdown;
- does not return raw provider payload JSON.

## 3. RunDetail DTO Shape

Top-level DTO:

```json
{
  "ticker": "string",
  "run_id": "string",
  "run_folder": "string",
  "status": {},
  "ai_usage": {},
  "provider": {},
  "company": {},
  "financial_interpretation": {},
  "blueprint": {},
  "self_review": {},
  "charts": [],
  "artifacts": {},
  "warnings": []
}
```

Nested fields implemented:

- `status`: `overall_status`, `provider_status`, `visual_lint_status`, `pdf_export_status`, `human_review_required`
- `ai_usage`: `source`, `external_ai_used`, `local_mock_used`, `cache_hits`, `new_external_ai_calls`, `model`, `prompt_versions`
- `provider`: `provider`, `source`, `provider_adapter`, `package_used`, `mock`, `market`, `currency`, `limitations`, `missing_fields`
- `company`: `name`, `identity`, `frame`, `not_this`, `confidence`
- `financial_interpretation`: `revenue_explanation`, `margin_explanation`, `cash_flow_explanation`, `where_money_comes_from`, `where_money_goes`, `debt_and_financing`, `valuation_method_fit`
- `blueprint`: `core_thesis`, `asset_profile`, `must_analyze`, `must_not_analyze_as_core`, `key_questions`, `red_flags`, `data_gaps`, `next_checks`
- `self_review`: `company_understanding_check`, `framework_fit_check`, `numeric_consistency_check`, `money_flow_check`, `final_confidence`, `human_review_required`
- `charts`: `title`, `image_path`, `source`, `status`, `explanation`
- `artifacts`: `markdown_report_path`, `pdf_report_path`, `dashboard_path`, `ai_usage_path`, `blueprint_path`, `validator_report_path`, `provider_payload_path`

## 4. Frontend Detail Behavior

When a user selects a run from the sidebar:

1. React records `selectedRunKey`.
2. The app calls `loadRunDetail(ticker, run_id)` through `studio/src/lib/tauri.ts`.
3. The UI shows a loading card.
4. On success, `activeRunDetail` is stored and rendered.
5. On failure, the UI shows an error card and does not crash.

Browser preview behavior:

- If Tauri IPC is unavailable, the UI shows a browser-preview warning.
- It does not pretend detail loading succeeded.

## 5. Cards Rendered

The main panel renders structured detail cards:

1. Header Card
2. AI Source Card
3. Provider Card
4. Company Identity Card
5. Money Flow Card
6. Blueprint Card
7. Data Gaps / Warnings Card

Charts are represented only by count and structured placeholders. Full chart rendering is intentionally not implemented.

## 6. Error Handling

Handled cases:

- invalid ticker or run id;
- path traversal;
- missing run folder;
- missing important metadata files;
- missing optional audit/chart files;
- malformed JSON;
- missing chart manifest;
- unavailable Tauri IPC in browser preview.

Missing important files are recorded in `warnings`. Missing optional files do not fail the command.

## 7. Tests Added

Rust/Tauri fixture tests added:

- `load_run_detail_rejects_path_traversal`
- `load_run_detail_missing_folder_returns_error`
- `load_run_detail_reads_required_metadata`
- `load_run_detail_handles_missing_optional_files`
- `load_run_detail_collects_warnings`
- `load_run_detail_detects_artifact_paths`
- `load_run_detail_does_not_return_raw_provider_payload`
- `load_run_detail_parses_ai_usage_summary`

The existing app info and list runs tests continue to pass.

## 8. Validation Commands Run

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

## 9. Validation Results

- `npm install`: PASS, with 2 moderate npm audit advisories reported by npm.
- `npm run typecheck`: PASS.
- `npm run build`: PASS.
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 16 tests passed.
- `cargo test --manifest-path research-rs/Cargo.toml`: PASS, 152 tests passed.
- `git diff --check`: PASS.

No external OpenAI API, provider network, training run, or v5 report generation was used.

## 10. Intentionally Not Implemented

Phase 5 intentionally does not implement:

- full markdown report rendering;
- artifact opening;
- D3;
- PDF;
- regression matrix;
- dashboard completion;
- provider calls;
- external AI calls;
- v5 core analysis changes.

## 11. Unrelated Files Left Untouched

Left unstaged and untouched:

- `reports/release_checks/v5_0/a_share_sector_smoke_report.md`

This is unrelated to the v6 Phase 5 implementation.

## 12. Remaining Blockers

No blockers for Phase 5.

Known non-blocking note: npm reports 2 moderate dependency advisories. They were not force-fixed because that could introduce unrelated dependency churn.

## 13. Next Phase Recommendation

Phase 6 should add artifact-opening commands or a deeper read-only artifact index. Full report markdown rendering and chart rendering should remain separate phases.

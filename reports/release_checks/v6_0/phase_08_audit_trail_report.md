# v6 Tauri Studio Phase 8 Audit Trail Report

## 1. Files changed

Legacy cleanup commit:

- `reports/release_checks/v5_0/a_share_sector_smoke_report.md`

Phase 8 files:

- `src-tauri/src/lib.rs`
- `studio/src/components/RunDetailPanel.tsx`
- `studio/src/styles.css`
- `studio/src/types/app.ts`
- `reports/release_checks/v6_0/phase_08_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_08_audit_trail_report.md`

## 2. Legacy tail cleanup result

The lingering untracked file `reports/release_checks/v5_0/a_share_sector_smoke_report.md` was inspected and determined to be valid release-check evidence. It was committed separately before Phase 8:

```text
831dc01 test: add A-share sector smoke report
```

Phase 8 does not stage or modify that v5 file.

## 3. Audit trail DTO shape

`load_run_detail` now returns:

```ts
audit_trail: Array<{
  stage: string;
  label: string;
  status: "pending" | "running" | "pass" | "warning" | "fail" | "skipped" | "cached" | "unknown";
  source: string | null;
  message: string | null;
  artifact_path: string | null;
}>;
```

This is a static reconstruction from completed-run metadata. It is not a live event stream.

## 4. Stages implemented

The backend builds nine audit stages:

1. Provider Fetch
2. Locked Data Validation
3. AI Company Understanding
4. AI Financial Interpretation
5. AI Research Blueprint
6. Report Rendering
7. AI Self Review
8. Validator / Lint
9. PDF / Pack / Export

## 5. Status mapping

- `PASS` metadata maps to `pass`.
- `WARNING`, degraded, or human-review-like states map to `warning`.
- `FAIL` metadata maps to `fail`.
- Missing optional artifacts map to `warning` or `unknown`.
- AI stages show `cached` when AI cache hits are present.
- AI stages show `warning` when `local_mock_used=true`.
- Provider stages show `warning` when provider missing fields or data-gap limitations are present.

## 6. UI placement

Added `AuditTrailPanel` inside the Run Detail UI after Data Gaps / Warnings and before general Artifact Links.

The panel renders:

- vertical timeline
- compact stage rows
- status badges
- source text
- message text
- per-stage Open buttons when an artifact path exists

## 7. Validator alert behavior

The UI derives compact validator alerts from existing `RunDetail` fields only:

- loader warnings
- report status
- provider status
- visual lint status
- self-review checks
- provider missing fields
- blueprint data gaps
- AI local/mock status
- human-review flags

It does not parse full report markdown and does not invent alerts when the source fields are empty.

## 8. Artifact open integration

Audit stages with an `artifact_path` use the existing Phase 7 `open_artifact` IPC helper. No new artifact-opening command was added.

Browser preview behavior remains safe: if Tauri IPC is unavailable, the UI shows a warning and does not pretend the action succeeded.

## 9. Tests added

Rust/Tauri tests added:

- `load_run_detail_builds_audit_trail`
- `audit_trail_marks_missing_optional_stage_unknown_or_warning`
- `audit_trail_marks_ai_source_external`
- `audit_trail_marks_local_mock_warning`
- `audit_trail_includes_artifact_paths_when_present`
- `audit_trail_does_not_require_live_events`
- `audit_trail_marks_provider_data_gap_warning`
- `audit_trail_does_not_parse_full_report_markdown`

The tests use temp fixture run folders and do not depend on the real `reports/` directory.

## 10. Validation commands run

- `npm install`
- `npm run typecheck`
- `npm run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path research-rs/Cargo.toml`
- `git diff --check`

## 11. Validation results

- Frontend typecheck: PASS
- Frontend build: PASS
- Tauri cargo check: PASS
- Tauri cargo build: PASS
- Tauri tests: PASS, 31 passed
- Existing v5 Rust tests: PASS, 152 passed
- `git diff --check`: PASS

`npm install` still reports two moderate npm audit advisories in the current JavaScript dependency tree. This was already present during earlier studio phases and is not caused by Phase 8 application logic.

## 12. What is intentionally not implemented

- Live pipeline event stream
- D3 charts
- PDF export
- Regression Matrix
- Full Markdown rendering
- Artifact content preview
- Provider calls
- External OpenAI calls
- v5 analysis, prompt, validator, training, or report generation changes

## 13. Unrelated files left untouched

None after the separate legacy cleanup commit.

## 14. Remaining blockers

None for Phase 8.

## 15. Final status

PASS

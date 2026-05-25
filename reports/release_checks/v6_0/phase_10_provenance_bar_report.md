# v6 Tauri Studio Phase 10 Provenance Bar Report

## 1. Files changed

- `studio/src/App.tsx`
- `studio/src/styles.css`
- `reports/release_checks/v6_0/phase_10_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_10_provenance_bar_report.md`

No backend DTO change was needed.

## 2. Component created/refined

Refined the Phase 2 `BottomProvenanceBar` into a real RunDetail-driven provenance and data-gaps panel.

Supporting helpers added:

- `booleanLabel`
- `compactList`
- `ProvenanceList`
- `provenanceWarnings`

## 3. AI provenance fields displayed

The bottom bar now displays:

- AI source
- `external_ai_used`
- `local_mock_used`
- `new_external_ai_calls`
- `cache_hits`
- model
- prompt versions

It also provides an optional `Open AI Usage` action using the existing `open_artifact` IPC command when `metadata/ai_usage.json` exists.

## 4. Provider fields displayed

The provider column displays:

- provider
- source
- provider adapter
- package used
- mock
- market
- currency
- provider limitations

## 5. Data gaps/warnings displayed

The data-gaps column combines:

- `blueprint.data_gaps`
- `provider.missing_fields`
- `RunDetail.warnings`
- human-review flags
- local/mock warning
- provider mock warning
- unsupported-claim hints from existing status/self-review strings

It also provides an optional `Open Blueprint` action using the existing `open_artifact` IPC command when `metadata/research_blueprint.json` exists.

## 6. Empty/loading/ready states

- No selected run: shows `Select a run to inspect AI provenance and data gaps.`
- Loading run detail: shows `Loading run provenance...`
- Ready run: shows real AI/provider/data-gap fields.
- Missing values show `unknown`/`UNKNOWN` instead of invented data.

## 7. Warning badge behavior

Badges shown from current RunDetail state:

- `EXTERNAL_AI` when `external_ai_used=true`
- `LOCAL_MOCK` when `local_mock_used=true`
- `CACHE` when cache hits are present
- `HUMAN_REVIEW` when report status or self-review requires human review
- `PROVIDER_MOCK` when provider metadata says mock data was used
- `DATA_GAP`/`WARNING` when data gaps, missing fields, or warnings are present

Warnings are not hidden.

## 8. Validation commands run

- `npm install`
- `npm run typecheck`
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path research-rs/Cargo.toml`
- `git diff --check`

## 9. Validation results

- Frontend typecheck: PASS
- Frontend build: PASS
- Tauri cargo check: PASS
- Tauri cargo build: PASS
- Tauri tests: PASS, 36 passed
- Existing v5 Rust tests: PASS, 152 passed
- `git diff --check`: PASS

`npm install` still reports two moderate npm audit advisories in the current JavaScript dependency tree. This was already present in earlier studio phases and was not introduced by Phase 10 data display logic.

## 10. What is intentionally not implemented

- D3
- Sankey charts
- PDF export
- Regression Matrix
- chart generation
- full markdown rendering
- provider calls
- external OpenAI calls
- deep link navigation from the bottom bar
- v5 analysis, prompt, validator, training, or report generation changes

## 11. Unrelated files left untouched

None.

## 12. Remaining blockers

None for Phase 10.

## 13. Final status

PASS

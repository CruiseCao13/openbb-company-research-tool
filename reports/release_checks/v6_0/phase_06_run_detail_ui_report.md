# v6.0 Phase 06 Run Detail UI Cards Report

Final status: PASS

## 1. Files Changed

- `studio/src/App.tsx`
- `studio/src/components/RunDetailPanel.tsx`
- `studio/src/styles.css`
- `reports/release_checks/v6_0/phase_06_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_06_run_detail_ui_report.md`

No Tauri backend, v5 provider, AI prompt, validator, training loop, report generation, eval set, or generated run-folder logic was modified.

## 2. Components Created / Refactored

Created `studio/src/components/RunDetailPanel.tsx` and moved run-detail presentation out of `App.tsx`.

Components created/refactored:

- `RunDetailPanel`
- `HeaderCard`
- `AiSourceCard`
- `ProviderCard`
- `CompanyIdentityCard`
- `MoneyFlowCard`
- `BlueprintCard`
- `DataGapsCard`
- `EmptyRunDetailState`
- `DetailSection`
- `KeyValueRow`
- `BulletList`

`App.tsx` now keeps app-level state and Tauri IPC orchestration, while the detail panel owns visual hierarchy and rendering.

## 3. Cards Implemented

The ready state renders:

1. Header Card
   - ticker
   - run id
   - overall status
   - human review flag
   - provider status
   - visual lint
   - PDF status
   - chart count
   - run folder path

2. AI Source Card
   - source
   - external AI used
   - local mock used
   - new external calls
   - cache hits
   - model
   - prompt versions
   - local mock warning when applicable

3. Provider Card
   - provider
   - source
   - adapter
   - package used
   - mock
   - market/currency
   - limitations
   - mock/public endpoint warnings when applicable

4. Company Identity Card
   - company name
   - frame
   - confidence
   - identity
   - not-this boundaries

5. Money Flow Card
   - where money comes from
   - where money goes
   - debt / financing
   - valuation fit
   - cash-flow explanation
   - data-gap warning when money-flow fields are absent

6. Blueprint Card
   - core thesis
   - must analyze
   - must not analyze as core
   - next checks

7. Data Gaps / Warnings Card
   - data gaps
   - missing provider fields
   - loader warnings
   - human review warning

## 4. Visual Improvements

- Added clearer run hero treatment for selected ticker/run id.
- Added semantic status badges across cards.
- Added warning copy treatment for local mock, provider mock, public endpoint fallback, and missing money-flow data.
- Added compact key-value grids with mono labels.
- Added consistent section titles and bullet-list blocks.
- Improved density and scanning while keeping the dark industrial v6 token system.

No raw JSON, full markdown, D3, PDF, or artifact opening was introduced.

## 5. Interaction States

Handled states:

- no run selected: `EmptyRunDetailState`
- selected run loading: loading card
- error: error card
- browser preview: Tauri runtime warning
- ready: full run detail cards

Browser preview does not pretend real filesystem discovery or detail loading succeeded.

## 6. Validation Commands Run

Frontend:

```bash
npm install
npm run typecheck
npm run build
```

Tauri:

```bash
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

## 7. Validation Results

- `npm install`: PASS, with 2 moderate npm audit advisories reported by npm.
- `npm run typecheck`: PASS.
- `npm run build`: PASS.
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 16 tests passed.
- `cargo test --manifest-path research-rs/Cargo.toml`: PASS, 152 tests passed.
- `git diff --check`: PASS.

No external OpenAI API, provider network, training run, artifact opening, or report generation was used.

## 8. Intentionally Not Implemented

Phase 6 intentionally does not implement:

- artifact opening;
- full markdown report rendering;
- D3;
- PDF;
- regression matrix;
- dashboard completion;
- new backend fields;
- new run-folder parsing;
- v5 core logic changes.

## 9. Unrelated Files Left Untouched

Left unstaged and untouched:

- `reports/release_checks/v5_0/a_share_sector_smoke_report.md`

This file is unrelated to the v6 Phase 6 implementation.

## 10. Remaining Blockers

No blockers for Phase 6.

Known non-blocking note: npm reports 2 moderate dependency advisories. They were not force-fixed because that could introduce unrelated dependency churn.

## 11. Next Phase Recommendation

Phase 7 can add read-only artifact opening or a narrow artifact index. Full markdown rendering, D3 charts, PDF previews, and regression matrix should remain separate phases.

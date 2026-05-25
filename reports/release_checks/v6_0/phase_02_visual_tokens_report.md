# v6.0 Phase 02 Visual Tokens and Base Layout Report

Final status: PASS

## 1. Files Changed

- `studio/src/App.tsx`
- `studio/src/styles.css`
- `reports/release_checks/v6_0/phase_02_visual_tokens_report.md`

No v5 provider, AI prompt, validator, training, report renderer, eval set, or generated run folder logic was modified.

## 2. Design Tokens Implemented

The global Studio stylesheet defines the required v6 dark industrial tokens:

- `--canvas: #030712`
- `--surface: rgba(15, 23, 42, 0.5)`
- `--surface-solid: #0f172a`
- `--border: #1e293b`
- `--text-main: #e5e7eb`
- `--text-muted: #94a3b8`
- `--pass: #34d399`
- `--warning: #fb923c`
- `--fail: #f43f5e`
- `--info: #8fd3ff`

Typography uses `Inter, system-ui, sans-serif` for UI text and `JetBrains Mono, ui-monospace, monospace` for status, labels, and data-like text. The base font size is 13px, with tabular numbers enabled on status and code-like elements.

## 3. Layout Sections Implemented

- Full viewport app shell with a dark canvas background and subtle grid treatment.
- Left sidebar fixed at 320px with:
  - `Runs`
  - `No run selected`
  - `Waiting for run discovery`
- Main panel with:
  - `Research Run Detail`
  - Subtitle explaining future locked data, AI provenance, validator logs, and artifacts.
  - Placeholder cards for Report Status, AI Source, Company Identity, Money Flow, and Data Gaps.
- Top status strip with:
  - `Studio shell ready`
  - `No external API used`
  - `No run loaded`
  - optional IPC readout from the Phase 1 `ping_studio` command.
- Bottom provenance bar fixed to the layout at 180px with:
  - `Provenance & Data Gaps`
  - AI provenance placeholder.
  - Data gaps placeholder.

All content remains static and clearly marked as placeholder or no-run-loaded.

## 4. Components Created

`studio/src/App.tsx` now contains small static components:

- `AppShell`
- `Sidebar`
- `TopStatusStrip`
- `StatusBadge`
- `ResearchCard`
- `BottomProvenanceBar`
- `EmptyState`

`StatusBadge` supports the requested variants:

- `PASS`
- `WARNING`
- `FAIL`
- `DATA_GAP`
- `EXTERNAL_AI`
- `LOCAL_MOCK`
- `UNKNOWN`

These variants are demo/static only in Phase 2 and are not wired to real run data.

## 5. Tailwind / CSS Approach

Plain global CSS was used instead of adding Tailwind.

Reason: Tailwind was not already configured, and Phase 2 did not require a new styling dependency. Keeping the visual system in `studio/src/styles.css` avoided extra config churn and kept the scaffold stable.

## 6. Validation Commands Run

Frontend:

```bash
npm install
npm run typecheck
npm run build
```

Tauri:

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Existing v5 Rust workspace:

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
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path research-rs/Cargo.toml`: PASS, 152 tests passed.
- `git diff --check`: PASS.

No external OpenAI API, provider network, report data, or real run folder was used.

## 8. Intentionally Not Implemented

Phase 2 intentionally does not implement:

- real run folder discovery;
- `list_runs`;
- `load_run_detail`;
- reading report artifacts;
- D3 charts;
- PDF rendering;
- regression matrix;
- dashboard data loading;
- provider calls;
- external AI calls;
- modifications to v5 core analysis logic.

## 9. Remaining Blockers

No blockers for the Phase 2 static visual shell.

Known non-blocking note: npm reports 2 moderate dependency advisories. They were not force-fixed because that could introduce unrelated dependency churn in a scaffold phase.

## 10. Next Phase Recommendation

Phase 3 should add a typed DTO contract and read-only Tauri filesystem command design before loading any real run folders. The frontend should continue to render only typed DTOs and should not directly read the filesystem or call network APIs.

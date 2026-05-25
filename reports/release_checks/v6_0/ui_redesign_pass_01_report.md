# v6 UI Redesign Pass 01 - Research Workspace Redesign

Date: 2026-05-26
Branch: v5-rust-ai-blueprint
Final status: PASS

## Problems Addressed

- Overall proportion was adjusted from a wide debug panel layout to a desktop research workspace.
- Run Detail and Regression Matrix now use separate workspace contexts.
- Report, Dashboard, PDF, AI Usage, Validator Audit, Provider Payload, and Reveal Folder actions are promoted into a primary action dock near the top.
- The bottom Provenance bar was replaced with a collapsed diagnostics strip and expandable drawer.
- Charts are now reached through a top-level run detail tab with clearer empty/missing-image states.
- Regression Matrix now has a dedicated title, selector, summary cards, legend, stronger cells, issue panel, and selected ticker inspector.
- Browser preview states remain honest and do not pretend real Tauri IPC loaded data.

## Files Changed

- studio/src/App.tsx
- studio/src/components/RunDetailPanel.tsx
- studio/src/styles.css
- reports/release_checks/v6_0/screenshots/ui_redesign_runs_workspace.png
- reports/release_checks/v6_0/screenshots/ui_redesign_matrix_workspace.png

## New Information Architecture

### Left Navigation Rail

- Width reduced to roughly 252px.
- Contains app title, Runs / Matrix mode switch, compact run filter, and scrollable run list.
- Run item displays ticker, shortened run_id, status badge, AI source badge, market/provider line, and human review badge when present.

### Main Workspace

Run mode is now a Report Workspace:

- Top identity area shows ticker, company name when loaded, run_id, report status, AI source, provider/source, chart count, artifact count, warning count, and IPC state.
- Primary Action Bar is directly below the header and exposes Report / Dashboard / PDF / Folder / AI Usage / Validator / Provider Payload.
- Detail content is split into tabs:
  - Summary
  - Charts
  - Audit Trail
  - Data Gaps
  - Artifacts

Matrix mode is now a separate Regression Matrix Hub and no longer shows "Research Run Detail" language.

### Diagnostics Drawer

- Collapsed strip height is about 52px.
- Shows AI source, provider, warning count, data gap count, and human review status.
- Expandable drawer shows AI provenance, provider source, and data gaps/warnings.
- Drawer is below the primary workspace and does not cover action buttons.

## Layout Changes

- Main workspace and run list scroll independently.
- Bottom diagnostics no longer permanently consumes 180px.
- Summary cards are grouped by intent rather than one long mixed card grid.
- Artifact dock is visually prominent and disabled states explain missing artifacts through button state/title.
- Matrix cells were enlarged and strengthened for a quality-board feel.

## Report / Dashboard / PDF Visibility

The primary action dock includes:

- Open Report
- Open Dashboard
- Open PDF
- Reveal Folder
- Open AI Usage
- Open Validator Audit
- Open Provider Payload

All artifact actions still use the existing Tauri IPC commands. The frontend does not navigate to file:// paths or read the filesystem directly.

## Chart Visibility

- Charts are available from a top-level "Charts" tab.
- The tab shows chart count when known.
- Chart cards show title, status, image preview when available, source, explanation fields, limitation, and Open Chart button.
- Missing charts show a clear empty state rather than a broken image or blank panel.

## Diagnostics Drawer Behavior

- Collapsed strip is always visible in Run mode.
- Expanded drawer presents full provenance/data-gap detail without replacing the main content.
- LOCAL_MOCK, PROVIDER_MOCK, HUMAN_REVIEW, DATA_GAP, and WARNING states remain visible.

## Regression Matrix Changes

- Matrix mode is isolated from run-detail language.
- Header says "Regression Matrix Hub".
- Training run selector is prominent.
- Summary cards show total tickers, average quality, warnings, hard failures, and provider failures.
- Matrix includes a legend and selected-cell visual state.
- Right inspector shows selected ticker details, issues, hard failures, issue distribution, and matrix warnings.

## Validation Results

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS
- `git diff --check`: PASS

No external OpenAI API, provider network, or training run was invoked.

## Screenshots

Generated browser-preview screenshot:

- reports/release_checks/v6_0/screenshots/ui_redesign_runs_workspace.png

Screenshot limitation: this was captured through browser preview because automated Tauri desktop screenshot capture was not available in this pass. Browser preview correctly shows desktop-required IPC states and does not prove desktop data loading. Matrix and real run-detail screenshots require an interactive Tauri desktop session.

## Remaining UI Blockers

- Real desktop click-through still needs user/manual Tauri runtime review with actual loaded runs.
- Browser-preview screenshots cannot show real run-detail tabs, charts, or matrix data because Tauri IPC is intentionally unavailable in browser mode.
- Additional refinement may be useful after seeing this layout with a real 1440x900 and 1728x1117 desktop session.

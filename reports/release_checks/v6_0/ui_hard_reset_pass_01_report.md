# v6 UI Hard Reset Pass 01 Report

## Final Status

PASS

## Old Problems Addressed

- Rebuilt the loaded-run information hierarchy so Research Workspace no longer starts with dense text cards.
- Made Money Flow Sankey the central visual block for a selected run.
- Promoted Report, Dashboard, PDF, Charts, Audit, AI Usage, Provider Payload, and Folder actions into a visible primary action bar.
- Separated Landing, Research, and Matrix workspaces through explicit top-level mode state.
- Removed raw local paths from the main App Info display; full paths are only available inside Advanced paths.
- Improved run search with result count, clear control, and honest empty states.
- Made the Matrix page visually stronger with larger rounded cells and selected-cell glow.
- Added a first-class chart preview strip before the tabbed detail content.

## New Information Architecture

- Landing Workspace: hero-only entry state with no empty run-detail cards, no raw paths, and CTA buttons.
- Research Workspace: company header, primary action bar, full-size Sankey stage, chart preview strip, detail tabs, diagnostics drawer, and right insight rail.
- Matrix Workspace: independent Regression Matrix Hub page, not mixed with Research Run Detail.
- Settings Center: remains a modal/side sheet with language, density, motion, glass, and research-view controls.

## Loaded-Run Improvements

- Selected run opens in Research mode.
- Default selected-run tab is Overview, but the large Sankey renders before tabs.
- Company monogram remains visible in header and run list.
- Insight rail only appears in Research mode, avoiding Landing/Matrix confusion.

## Sankey Changes

- `MoneyFlowSankey` is now exported and rendered as the central graph stage.
- Stage minimum height is now approximately 400-430px, with SVG height increased to support a larger visual.
- Sankey remains honest: current data path is qualitative and explicitly labeled as not amount-scaled.
- Link shimmer, hover highlight, and reduced-motion support remain CSS-based.

## Action Bar Visibility

- Primary action bar is sticky near the top of the Research Workspace.
- Actions use short icon-like mono tokens plus labels.
- Missing artifacts are disabled.
- Success/failure messages no longer reveal full local paths.

## Path Hygiene

- Main UI avoids `/Users/...` path display.
- App Info shows Repo root and Reports root as available states.
- Full repo/report paths are available only under `Advanced paths`.
- Artifact actions use semantic labels such as Report, Dashboard, PDF, Provider Payload, and Folder.

## Language And Settings

- Added/updated bilingual UI keys for Research, Overview, Search runs, Clear, Open Charts, Open Audit, Chart Gallery, and View Quality Matrix.
- Language switch remains in top chrome.
- Settings Center remains session-persistent through localStorage.

## Search Changes

- Sidebar search is explicitly labeled Search runs.
- Result count displays filtered / total run count.
- Clear button appears when a query is active.
- No global command palette is implied.

## Matrix Changes

- Matrix remains a separate workspace.
- Cells are larger, consistently rounded, and have stronger hover/selected states.
- Legend, filters, issue distribution, and selected ticker inspector remain in place.

## Chart Gallery Changes

- Added a chart preview strip above tabs in Research Workspace.
- Shows chart count, top chart previews, source labels, and missing-image states.
- Full chart gallery remains in the Charts tab and only uses existing artifacts.

## Liquid Glass Implementation

- Added reusable classes: `glass-shell`, `glass-panel`, `glass-card`, `glass-control`, `glass-button`, `glass-badge`.
- Applied liquid-glass treatment to shell surfaces: rail, topbar, hero, action bar, graph stage, chart strip, matrix, settings, and diagnostics.
- Radius hierarchy follows shell > panel > card > control > pill.

## Motion Implementation

- Mouse-following glow remains active through shell CSS variables.
- Landing has light-wave/orbit motion.
- Warning/fail/data-gap/local/mock badges use subtle breathing animation.
- Sankey links shimmer.
- Run items, matrix cells, cards, and chart previews lift/glow on hover.
- `prefers-reduced-motion` and Studio motion setting disable animation.

## Screenshots

- Generated: `reports/release_checks/v6_0/screenshots/ui_hard_reset_pass_01_landing.png`
- Browser-preview screenshot only. Loaded-run Tauri runtime screenshot remains a manual desktop QA item because headless browser cannot call Tauri IPC.

## Validation

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 44 tests passed
- `git diff --check`: PASS
- Secret scan for real `sk-...` keys: PASS, no real key pattern found

## Remaining Blockers

- Loaded-run screenshot still needs real Tauri desktop verification.
- Sankey remains qualitative until normalized numeric money-flow fields are exposed in the existing DTO.
- Settings mode is implemented as a modal, not a separate routed full page.
- Chart previews depend on existing chart artifacts; missing chart artifacts show honest empty states.

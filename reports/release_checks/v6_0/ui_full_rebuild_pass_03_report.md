# v6 UI Full Rebuild Pass 03 Report

## Final Status

WARNING

The UI now passes build and validation, uses benchmark-derived patterns, and is much more graph-first. Status remains WARNING because loaded-run screenshots still require real Tauri runtime QA, Sankey remains qualitative until numeric money-flow DTO fields are exposed, and Run Local Analysis is intentionally deferred rather than faked.

## Benchmark Ideas Used

- OpenBB Workspace: finance workspace as visual analysis surface, not file browser.
- FinceptTerminal: professional terminal density and instrumentation.
- shadcn command/search patterns: search as primary action surface.
- D3 Sankey: use Sankey for layout while customizing SVG rendering.
- Apple materials: translucent layered surfaces with hierarchy.
- OpenAI-style landing: minimal copy, strong central visual, focus over clutter.

Detailed benchmark notes are in `reports/release_checks/v6_0/ui_benchmark_mining_pass_03.md`.

## Old UI Failures Addressed

- Landing no longer promotes random run folders as the main content.
- Run list becomes secondary and example-first until a search is entered.
- Search now controls user intent and shows no-result feedback.
- Vascular flow visual is the primary landing object.
- Research workspace keeps Money Flow central and large.
- Gauges replace plain status cards in the right rail.
- Dense text moves behind hover or supporting tabs where feasible.
- Raw paths remain hidden from primary UI.

## New Information Architecture

1. Landing: visual demo, example chips, search/analyze console.
2. Search: ticker/company input, market selector, mode selector, Analyze, Load Latest, View Matrix.
3. Research: company header, action toolbar, large Sankey, chart strip, tabs, gauges, diagnostics.
4. Matrix: separate quality-board workspace.
5. Settings: Liquid Glass modal with persisted controls.

## Landing Behavior

- Displays Example Preview with AAPL, GOOGL, RKLB, 600519.SH, and JPM.
- Shows a large animated vascular money-flow demo.
- Provides a central search/analyze console.
- Marks external AI as disabled.
- Does not show local paths, logs, or raw metadata.

## Search Behavior

- Search filters known run metadata.
- Enter/Analyze loads the latest matching existing run.
- If no matching run exists, the user stays on landing and sees “No existing run found” plus Run Local Analysis coming-next copy.
- Load Latest respects the search query when present.
- No fake run execution is implemented.

## Loaded-Run Behavior

- Research mode defaults to Flow.
- Money Flow remains the main visual object.
- Report, Dashboard, PDF, Charts, Audit, AI Usage, Provider Payload, and Folder actions stay visible in the toolbar.
- Diagnostics remain collapsed by default.

## Sankey Behavior

- Sankey uses `d3-sankey` layout and custom SVG rendering.
- Added thicker vascular links, glow filter, gradients by flow type, stroke shimmer, capsule nodes, and hover label reveal.
- Qualitative mode is clearly labeled as not amount-scaled.
- No numeric values are fabricated.

## Gauge Dashboard

Gauges now show:

- Data Confidence
- Cash Flow Quality
- Money Flow Specificity
- Human Review
- Provider Coverage

Gauge explanations expand on hover. They use status-based color and do not imply unavailable precision.

## Path Hygiene

- No `/Users/...` path is used in primary landing or research surfaces.
- Full paths remain available only in Advanced paths or artifact commands.
- Artifact actions use semantic labels.

## Language / Settings

- Added bilingual labels for the new landing console, examples, gauges, tabs, no-result state, and settings headings.
- Settings remain persisted via localStorage.
- Some generated metadata strings remain intentionally untranslated.

## Matrix Board

- Matrix remains a separate workspace.
- Larger rounded cells, filters, legend, summary, issue distribution, and inspector remain in place.

## Charts

- Chart previews stay in the research workspace.
- Chart gallery hides dense text until hover.
- Missing chart images show a designed empty state.

## Motion

- Cursor-following glow remains active.
- Landing vascular flow and Sankey links animate.
- Gauge, matrix, chart, warning, settings, and diagnostics interactions use restrained motion.
- Reduced-motion behavior is preserved.

## Screenshots

- Generated: `reports/release_checks/v6_0/screenshots/ui_full_rebuild_pass_03_landing.png`
- Loaded-run, Sankey runtime, matrix, settings, diagnostics, and charts screenshots require Tauri desktop QA because browser preview cannot invoke Tauri IPC.

## Validation

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 44 tests passed
- `git diff --check`: PASS
- Real secret scan for `sk-...` keys: PASS

## Remaining Blockers

- Need a future `v6 Quantitative Money Flow DTO Pass` for amount-scaled Sankey.
- Need real Tauri desktop screenshots for loaded-run and settings flows.
- Run Local Analysis command wrapper remains deferred.
- The UI still uses some monolithic `App.tsx` structure; a future component extraction pass would make it easier to maintain.

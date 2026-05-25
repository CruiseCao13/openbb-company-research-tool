# v6 UI Hard Reset Pass 02 Report

## Final Status

WARNING

The UI pass builds and validates, and the landing/research architecture is now graph-first. Status remains WARNING because the Sankey is still qualitative due missing numeric money-flow DTO fields, loaded-run screenshots require a real Tauri desktop runtime, and Run Local Analysis remains intentionally deferred instead of faked.

## Old Problems Addressed

- Removed run-list dominance from the landing state.
- Replaced the homepage debug/list feeling with a visual vascular money-flow preview.
- Added an explicit ticker search/analyze console with market and mode selectors.
- Limited landing examples to AAPL, GOOGL, RKLB, 600519.SH, and JPM, clearly marked as Example Preview.
- Reworked the selected-run view so the central visual language is Money Flow rather than text cards.
- Added gauge-style diagnostics in the right rail.
- Kept local paths out of the primary UI.
- Hid chart explanations behind hover interaction instead of dumping text by default.

## New Homepage Behavior

- Landing shows a large animated vascular flow preview and short opening copy.
- Landing search supports ticker/company input, market selector, and mode selector.
- External AI mode is disabled in UI and not invoked.
- Run Local Analysis is shown as coming next, with no fake execution.
- The left rail shows example companies until the user searches.

## Loaded-Run Behavior

- Research Workspace still uses existing Tauri IPC data only.
- Company header, primary action bar, large Sankey, chart preview strip, tabs, gauges, and diagnostics remain separate.
- Default selected-run tab is Flow.
- Long explanatory text is pushed into hover/details areas where feasible.

## Sankey Design

- Rebuilt the central Sankey as a vascular/river-like glowing flow network.
- Added SVG glow filters, multiple gradients, thicker rounded links, node capsules, and shimmer animation.
- Labels are subdued by default and brighten on hover.
- Qualitative mode is explicitly labeled: width is not amount-scaled.
- No dollar amounts or numeric widths are invented.

## Gauge Dashboard Behavior

- Added gauge-style components for:
  - Data Confidence
  - Cash Flow Quality
  - Money Flow Specificity
  - Human Review
  - Provider Coverage
- Gauges use status-based rings instead of fake precision.
- Explanations expand on hover.

## Language Behavior

- Added bilingual UI chrome for landing search, examples, modes, gauges, settings basics, tabs, and chart/matrix labels touched by this pass.
- Tickers, artifact names, and generated report contents remain untranslated.

## Settings Behavior

- Settings remain a Liquid Glass modal and continue persisting via localStorage.
- Language, density, motion, glass, font scale, warnings-first, default landing, start latest, and matrix default controls still apply immediately.
- External AI is not enabled by settings in this pass.

## Search / Analyze Flow

- Search filters known runs and controls landing intent.
- Enter/Analyze loads the latest matching existing run if available.
- Load Latest loads the newest matching run when a query exists, otherwise the newest run.
- If no matching run exists, the UI stays honest and does not fabricate a result.

## Chart Gallery

- Chart preview strip remains near the top of Research Workspace.
- Chart cards hide dense explanations until hover.
- Missing chart images render as designed empty states, not broken images.

## Matrix Board

- Matrix remains a separate workspace.
- Existing larger rounded cells, hover glow, selected ring, filters, issue distribution, and inspector remain intact.

## Path Hygiene

- No full local path is shown in the landing or primary research visual.
- App repo/report paths are available only through Advanced paths.
- Artifact actions use semantic labels.

## Debug Hiding

- Run lists do not dominate landing.
- Metadata/log style content is relegated to diagnostics, advanced details, tabs, or hover states.
- Primary surfaces use visual badges, gauges, and graph states.

## Screenshots

- Generated: `reports/release_checks/v6_0/screenshots/ui_hard_reset_landing.png`
- Loaded-run screenshots were not captured because browser preview cannot call Tauri IPC; this needs desktop runtime QA.

## Validation

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 44 tests passed
- `git diff --check`: PASS
- Real secret scan for `sk-...` keys: PASS

## Remaining Blockers

- Need `v6 Quantitative Money Flow DTO Pass` to expose numeric revenue/OCF/capex/FCF/debt/dividend/buyback fields for amount-scaled Sankey.
- Need real desktop runtime screenshots for loaded-run, Sankey, charts, matrix, settings, and diagnostics states.
- Run Local Analysis from UI is intentionally disabled and should be implemented later as a safe local-only command wrapper.

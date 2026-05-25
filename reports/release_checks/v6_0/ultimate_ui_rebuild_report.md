# v6 Ultimate UI Rebuild Report

## Benchmark Sources Inspected

- OpenBB: finance workspace/product architecture. No code copied.
- FinceptTerminal: finance-terminal density and multi-tool posture. No code copied.
- shadcn-admin: sidebar, settings, and command/search organization patterns. No code copied.
- d3-sankey: Sankey graph layout vocabulary and existing API usage. No source copied.

See `reports/release_checks/v6_0/ui_benchmark_mining_ultimate.md`.

## Files Changed

- `studio/src/App.tsx`
- `studio/src/i18n.ts`
- `studio/src/styles.css`
- `studio/src/components/LandingExperience.tsx`
- `studio/src/components/InstrumentBoard.tsx`
- `studio/src/components/LogoMark.tsx`
- `reports/release_checks/v6_0/ui_benchmark_mining_ultimate.md`
- `reports/release_checks/v6_0/ultimate_ui_rebuild_report.md`
- `reports/release_checks/v6_0/screenshots/ultimate_ui_landing.png`
- `reports/release_checks/v6_0/screenshots/ultimate_ui_matrix.png`
- `reports/release_checks/v6_0/screenshots/ultimate_ui_settings.png`

## Components Rewritten

- Extracted landing/search/demo flow into `LandingExperience`.
- Extracted company identity tile into `LogoMark`.
- Extracted right-side gauge/instrument rail into `InstrumentBoard`.
- Removed the primary-header IPC ping readout so loaded-run UI no longer surfaces debug status in the main company header.

## Landing Result

- Landing is now a visual entry surface with a large qualitative vascular flow preview, example company chips, and a search/analyze console.
- The landing preview is explicitly labeled as an example and qualitative.
- No raw local paths, logs, or metadata dumps are shown on the landing screen.

## Search Result

- Search remains the central landing interaction.
- It filters the run rail, supports Enter/analyze to load matching existing runs, and shows a no-existing-run state.
- Run Local Analysis and External AI remain disabled/deferred; no fake run execution is exposed.

## Loaded-Run Result

- Loaded-run architecture remains graph-first: company header, primary action dock, large Money Flow visual, chart strip, tabs, and diagnostics.
- Report, Dashboard, PDF, Charts, Audit, AI Usage, Provider Payload, and Folder actions remain visible near the top.
- Browser preview cannot load real run artifacts because Tauri IPC is unavailable outside the desktop runtime.

## Sankey Result

- The Money Flow visual remains central and uses existing `d3-sankey`.
- The UI explicitly labels the current graph as qualitative when numeric locked money-flow values are not available in the DTO.
- Link widths are not presented as financial amounts; no numeric values are fabricated.

## Gauge Result

- The right rail is now an instrument board with gauges for data confidence, cash-flow quality, money-flow specificity, provider coverage, human review, valuation risk, and template leakage.
- Gauges derive qualitative status from existing RunDetail fields and show Unknown/Data Gap when a score is not exposed.

## Matrix Result

- Matrix remains a separate workspace and is not mixed into the research-run view.
- Browser preview shows the honest Tauri-runtime-required state because `list_training_runs` requires IPC.
- Real matrix board behavior remains dependent on desktop runtime verification.

## Chart Result

- The chart gallery remains available as a first-class tab and preview strip.
- Missing chart images render as an intentional empty/warning state, not as broken images.

## Path Hygiene

- Full local filesystem paths are not used as primary labels.
- Artifact actions use product labels such as Report, Dashboard, PDF, Provider Payload, and Folder.
- Advanced app paths remain behind collapsed advanced details.

## Language / Settings

- EN/中文 chrome was tightened for the landing, search, run, artifact, gauge, and settings labels.
- The landing vascular demo node labels are now translated.
- Settings remain in-session controls for language, density, motion, glass intensity, font scale, warnings-first, landing default, and matrix default.

## Motion / Glass

- Liquid Glass utility classes were expanded for canvas, spatial zones, floating surfaces, gauges, drawers, inspectors, action pills, mono labels, metric values, and status orbs.
- Main Sankey stage sizing was increased and retains animated vascular flow styling with reduced-motion support.
- Cursor glow, light-wave background, hover lift, warning pulse, gauge hover, matrix hover, and settings/drawer transitions remain CSS-first.

## Screenshots

- Captured: `reports/release_checks/v6_0/screenshots/ultimate_ui_landing.png`
- Captured: `reports/release_checks/v6_0/screenshots/ultimate_ui_matrix.png`
- Captured: `reports/release_checks/v6_0/screenshots/ultimate_ui_settings.png`
- Blocked: loaded-run, real Sankey with run artifacts, chart tab with artifacts, and real quality matrix screenshots require Tauri IPC desktop runtime rather than browser preview.

## Validation

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 44 tests passed
- `git diff --check`: PASS
- Secret scan for real `sk-` keys: PASS

## Remaining Blockers

- Quantitative Sankey requires explicit numeric money-flow fields in the v6 DTO. Current UI stays qualitative rather than inventing values.
- Browser preview cannot prove loaded-run runtime behavior because artifact commands and list/load IPC require Tauri desktop runtime.
- Run Local Analysis from the UI is intentionally deferred.

## Final Status

WARNING

Reason: frontend build passes and the UI is graph-first with no fake data, but runtime screenshots for loaded runs and matrix artifacts require Tauri desktop IPC, and the current Sankey remains qualitative pending a quantitative money-flow DTO.

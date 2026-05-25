# v6 UI Visual Direction Spike 01

## Scope

This was a low-cost visual direction spike, not a full productization pass. It kept the existing Tauri IPC contracts and did not touch research logic, providers, training, validators, prompts, generated run folders, report generation, or financial calculations.

## Files Changed

- `studio/src/App.tsx`
- `studio/src/i18n.ts`
- `reports/release_checks/v6_0/ui_visual_direction_spike_01.md`

## Visual Direction Implemented

- The current v6 Studio direction is graph-first, black-gold/deep-space, liquid-glass, and borderless rather than file-browser or debug-dashboard oriented.
- Landing already uses the minimal "See with Your Own Eyes" / "亲眼看见" visual entry with example tickers and a vascular preview.
- Loaded-run state is already centered on the large money-flow visual, with company identity, artifact actions, chart strip, gauge/insight rail, and collapsed diagnostics.
- This spike tightened visible language chrome in the graph-first workspace so the top run signals, action feedback, loading flow state, and chart preview strip no longer fall back to hard-coded English in Chinese mode.

## Landing Result

- Landing remains a visual entry rather than a run-list or metadata panel.
- It does not show raw logs, local paths, generated run folders, or long report text as primary content.
- Example preview remains qualitative and does not claim to be live research output.

## Loaded-Run Result

- Loaded-run workspace remains graph-first: company header, primary action dock, large Sankey stage, chart strip, detail tabs, instrument board, and secondary diagnostics.
- Primary actions remain obvious: Report, Dashboard, PDF, Charts, Audit, AI Usage, Provider Payload, and Folder.
- This spike localized the remaining visible status strings around provider/market fallback, chart/artifact/warning counters, chart-empty states, artifact action feedback, and graph loading copy.

## Sankey Result

- Money Flow Sankey remains large and central in the loaded-run workspace.
- It remains honest qualitative mode when numeric money-flow DTO fields are unavailable.
- The UI continues to state qualitative flow behavior rather than inventing amount-scaled widths.

## Path Hygiene Result

- No full `/Users/...` local path is used as primary UI text.
- Artifact controls use product labels rather than raw filesystem paths.
- Paths remain limited to advanced/debug contexts.

## Deferred

- No new backend DTO fields were added for quantitative Sankey.
- No full matrix redesign was done in this spike.
- No settings, command menu, split-pane, tooltip, or audio work was expanded in this spike because those are outside the low-cost visual direction goal.
- Screenshot capture was not regenerated in this spike to keep the pass lightweight; prior v6 screenshots remain under `reports/release_checks/v6_0/screenshots/`.

## Validation

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 44 tests passed
- `git diff --check`: PASS

No provider, external AI, research, or training runs were executed.

## Screenshot Status

WARNING: no new `ui_visual_spike_*` screenshots were generated in this low-cost spike. Existing visual evidence remains in:

- `reports/release_checks/v6_0/screenshots/ultimate_ui_command_landing.png`
- `reports/release_checks/v6_0/screenshots/ultimate_ui_command_matrix.png`
- `reports/release_checks/v6_0/screenshots/ultimate_ui_command_settings_audio.png`

## Final Status

WARNING

Reason: the visual direction is already implemented and this spike produced a concrete language/visual-chrome cleanup plus a focused validation report, with frontend typecheck and build passing. Status remains WARNING because Sankey is qualitative, no new screenshots were captured, and this pass intentionally did not attempt the deferred full-product items.

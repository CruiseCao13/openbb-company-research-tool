# v6 Tauri Studio Phase 13: Studio Alpha Acceptance Review

Final status: PASS

## Files Changed

- `reports/release_checks/v6_0/phase_13_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_13_studio_alpha_acceptance_report.md`

No Studio source or Tauri command changes were required during this acceptance pass.

## Functionality Checklist

- App shell loads at build time: PASS
- `get_app_info` displays through typed IPC: PASS
- `list_runs` loads existing v5 run folders through typed IPC: PASS
- Selecting a run calls `load_run_detail`: PASS
- Run detail cards are present: PASS
- Audit trail panel is present: PASS
- Chart grid from existing artifacts is present: PASS
- Provenance and data gaps bottom bar is present: PASS
- Artifact buttons use backend `open_artifact` / `reveal_in_folder`: PASS
- Regression Matrix mode is present: PASS
- Training run selector and quality matrix cells are present: PASS
- Matrix selected ticker detail is present: PASS
- Empty states are implemented for runs, detail, charts, and matrix: PASS
- Error states are implemented for IPC failures: PASS
- Browser preview warning is implemented when Tauri IPC is unavailable: PASS
- Frontend direct filesystem access scan: PASS
- Frontend external network access scan: PASS
- Secret exposure scan: PASS

## UX Consistency Audit

- PASS / WARNING / FAIL status badges use consistent semantic treatments: PASS
- Human review required is visible in detail and bottom bar: PASS
- `local_mock_used=true` warning is visible: PASS
- Provider mock warning is visible: PASS
- External AI provenance badge is visible when present: PASS
- Data gaps are visible in detail cards and bottom bar: PASS
- Selected run has a clear sidebar highlight: PASS
- Selected training run uses the matrix selector: PASS
- Chart missing state uses a warning card, not a broken image: PASS
- Missing artifact buttons are disabled: PASS
- Regression matrix color mapping matches score bands: PASS
- Bottom bar is fixed in the app grid and does not replace main cards: PASS
- Main panes and run lists use scroll containers: PASS
- Raw JSON is not dumped in the UI: PASS
- Placeholder text is limited to no-run/no-data/browser-preview states: PASS

## Safety Audit

- `open_artifact` rejects path traversal and outside-report paths: PASS
- `reveal_in_folder` rejects outside-report paths: PASS
- `load_run_detail` rejects unsafe ticker/run-id segments: PASS
- `load_quality_matrix` rejects unsafe run-id segments: PASS
- No API key display path found in Studio frontend or Tauri source: PASS
- `.env` is not read by the frontend: PASS
- Studio UI does not call providers or external AI: PASS
- Run folders and training artifacts are read-only from Studio commands: PASS
- Artifact commands validate paths under `repo_root/reports`: PASS

## Manual Data Path Smoke

Existing local artifacts were checked without generating runs or calling providers/API:

- `AAPL`: 84 run folders available.
- `RKLB`: 37 run folders available.
- `600519.SH`: 19 run folders available.
- `000001.SZ`: 13 run folders available.
- `reports/training_runs`: 8 training run folders available.

This confirms Phase 4/5/12 discovery has local paths to exercise in the desktop runtime. No new report, provider, AI, or training execution was performed.

## Fixes Made

No code fixes were required. Phase 13 generated acceptance documentation only.

## Validation Commands Run

- `npm install` - PASS, restored local frontend dependencies for validation. It reported two moderate npm audit advisories; no dependency changes were made.
- `npm run typecheck` - PASS
- `npm run build` - PASS
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` - PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` - PASS
- `cargo build --manifest-path src-tauri/Cargo.toml` - PASS
- `cargo test --manifest-path src-tauri/Cargo.toml` - PASS, 44 tests passed
- `cargo test --manifest-path research-rs/Cargo.toml` - PASS
- `python3 -m py_compile providers/*.py` - PASS
- `git diff --check` - PASS
- Real secret scan for `sk-...` and `Authorization: Bearer sk-` - PASS

Generated validation artifacts (`node_modules`, `studio/dist`, and `src-tauri/target`) were removed before staging.

## Known Limitations

- Browser preview cannot perform real filesystem-backed discovery because Tauri IPC is unavailable there; the UI shows explicit runtime warnings.
- Regression Matrix Hub reads existing matrix artifacts only; it does not run training or generate matrices.
- Chart Grid reads existing PNG/manifest artifacts only; it does not generate charts or D3 views.
- Artifact opening delegates to OS behavior through Tauri and remains limited to files under `repo_root/reports`.
- No full Markdown report rendering is implemented inside Studio.
- No PDF export is implemented inside Studio.

## Next Recommended Phase

Phase 14 should be a focused desktop runtime QA pass: launch the Tauri app, click through representative AAPL, RKLB, `600519.SH`, `000001.SZ`, and one training matrix run, then capture screenshots or notes for any remaining layout issues. It should still avoid v5 logic, provider, prompt, training, and report-generation changes.

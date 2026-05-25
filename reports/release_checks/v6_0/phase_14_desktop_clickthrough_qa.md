# v6 Tauri Studio Phase 14 Desktop Click-through QA

Date: 2026-05-25
Branch: `v5-rust-ai-blueprint`
Scope: Desktop runtime QA, small runtime fixes, and acceptance report only.

## Environment

- Workspace: `/Users/cruise/Projects/investment-tools/openbb-company-research-tool`
- Runtime attempted: Tauri v2 desktop dev runtime and bundled macOS app
- Frontend: Vite / React / TypeScript
- External AI: not used
- Provider network: not used
- Training runs: not executed

## Runs Tested

The following existing run directories were available for QA targeting:

- `reports/AAPL/runs/`
- `reports/RKLB/runs/`
- `reports/600519.SH/runs/`
- `reports/000001.SZ/runs/`

Desktop click-through selection for these runs could not be completed because the Tauri desktop window was not visible/clickable in the current macOS runtime session after launch. This is recorded as a runtime QA blocker, not as a product pass.

## Training Matrix Tested

Existing training run artifacts were present under `reports/training_runs/`, including quality matrix artifacts such as:

- `reports/training_runs/external_regression_pass_01/quality_matrix.json`
- `reports/training_runs/broad_500_dryrun_10/quality_matrix.json`

Regression Matrix desktop click-through could not be completed because of the same desktop window visibility blocker.

## Interaction Checklist

| Check | Result | Notes |
| --- | --- | --- |
| App starts | WARNING | Initial launch panicked due an invalid icon. After the icon fix, the process launches, but the window is not observable/clickable. |
| AppInfo card loads | NOT VERIFIED DESKTOP | Browser preview now shows an honest Tauri IPC unavailable warning instead of crashing. |
| Run list loads | NOT VERIFIED DESKTOP | Existing typed IPC tests/builds pass; desktop click-through blocked by invisible window. |
| Selecting AAPL loads detail | NOT VERIFIED DESKTOP | Existing run folders are present. |
| Selecting RKLB loads detail | NOT VERIFIED DESKTOP | Existing run folders are present. |
| Selecting 600519.SH loads detail | NOT VERIFIED DESKTOP | Existing run folders are present. |
| Selecting 000001.SZ loads detail | NOT VERIFIED DESKTOP | Existing run folders are present. |
| AI Source card displays provenance | NOT VERIFIED DESKTOP | Covered by existing UI/type validation, but not click-verified. |
| Provider card displays source/package/mock/currency/market | NOT VERIFIED DESKTOP | Covered by existing UI/type validation, but not click-verified. |
| Data gaps visible | NOT VERIFIED DESKTOP | Covered by existing UI/type validation, but not click-verified. |
| Audit trail visible | NOT VERIFIED DESKTOP | Covered by existing UI/type validation, but not click-verified. |
| Chart grid visible | NOT VERIFIED DESKTOP | Covered by existing UI/type validation, but not click-verified. |
| Artifact buttons open/reveal valid files | NOT VERIFIED DESKTOP | Existing backend safety tests pass; desktop clicking blocked. |
| Regression Matrix loads training run | NOT VERIFIED DESKTOP | Existing artifacts are present; desktop clicking blocked. |
| Bottom provenance bar does not block content | NOT VERIFIED DESKTOP | Needs visible desktop window for final confirmation. |
| Browser preview states are honest | PASS | Fixed raw `invoke` TypeError into an explicit browser-preview/Tauri IPC unavailable warning. |
| No direct filesystem/network access from frontend | PASS | Source scan found no direct frontend file/network calls. |
| No secret exposure | PASS | Real secret scan found no API key pattern. |

## Bugs Found

1. The Tauri runtime panicked at startup because `src-tauri/icons/icon.png` was invalid/empty for the declared icon dimensions.
2. After the icon fix, the Tauri process launched but no desktop window was visible/clickable in the current macOS session.
3. Browser preview mode showed raw JavaScript errors for unavailable Tauri IPC instead of an honest preview warning.
4. `npm run tauri:build` produced a macOS `.app` bundle but failed during DMG bundling. The required Phase 14 validation commands do not depend on DMG creation, but this remains a packaging limitation.

## Fixes Made

- Replaced `src-tauri/icons/icon.png` with a valid 32x32 PNG icon.
- Added a conservative Tauri setup hook that ensures the main window exists and requests show/focus.
- Added explicit window URL and initial position in `src-tauri/tauri.conf.json`.
- Enabled bundle icon configuration for the Tauri app.
- Centralized frontend Tauri IPC calls behind a helper that reports browser-preview unavailability without crashing.
- Made chart artifact image conversion return an empty preview path when Tauri file conversion is unavailable in browser preview.

## Screenshots

One compressed screenshot was retained to document the desktop visibility blocker:

- `reports/release_checks/v6_0/screenshots/phase_14_window_visibility_blocker.jpg`

The screenshot shows the desktop session after launch attempts with no visible Tauri Studio window.

## Validation Results

| Command | Result |
| --- | --- |
| `npm run typecheck` | PASS |
| `npm run build` | PASS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | PASS |
| `cargo check --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo build --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS, 44 tests |
| `cargo test --manifest-path research-rs/Cargo.toml` | PASS |
| `python3 -m py_compile providers/*.py` | PASS |
| `git diff --check` | PASS |
| Real secret scan | PASS |

## Final Status

Final status: WARNING

The Phase 14 build/test/safety checks pass, and two concrete runtime bugs were fixed. However, the actual desktop click-through acceptance is not complete because the Tauri desktop window could not be observed or clicked in the current macOS runtime session. The next phase should focus narrowly on resolving Tauri window visibility/activation and repeating the click-through checklist before declaring desktop QA PASS.

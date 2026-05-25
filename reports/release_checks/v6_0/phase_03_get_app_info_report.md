# v6.0 Phase 03 get_app_info IPC Report

Final status: PASS

## 1. Files Changed

- `src-tauri/src/lib.rs`
- `studio/src/App.tsx`
- `studio/src/components/AppInfoCard.tsx`
- `studio/src/lib/tauri.ts`
- `studio/src/types/app.ts`
- `studio/src/styles.css`
- `reports/release_checks/v6_0/phase_03_get_app_info_report.md`

No v5 provider, AI prompt, validator, training loop, report generation, financial calculation, eval set, or generated run-folder logic was modified.

## 2. Command Implemented

Implemented Tauri IPC command:

```rust
get_app_info() -> Result<AppInfo, String>
```

The Rust backend owns repo-root discovery by walking up from the current directory and checking for the expected v5/v6 workspace markers:

- `research-rs/Cargo.toml`
- `studio/index.html`
- `src-tauri/Cargo.toml`

The frontend does not read the filesystem and does not use `fetch` for local files.

## 3. DTO Shape

The command returns:

```json
{
  "app_version": "v6.0-alpha",
  "repo_root": "/path/to/openbb-company-research-tool",
  "reports_root": "/path/to/openbb-company-research-tool/reports",
  "platform": "macos",
  "studio_mode": "shell"
}
```

Field meanings:

- `app_version`: fixed Phase 3 shell version string.
- `repo_root`: backend-discovered repo root.
- `reports_root`: `repo_root/reports`.
- `platform`: Rust `std::env::consts::OS`.
- `studio_mode`: `shell`.

## 4. Frontend Display Added

Added `AppInfoCard` to the existing Phase 2 layout. It displays:

- App version
- Repo root
- Reports root
- Platform
- Studio mode
- IPC status

Loading/error states:

- `Loading app info...`
- `IPC connected`
- `IPC failed`
- `Fallback dev warning` when Tauri IPC is unavailable in browser preview.

Command failures are shown in the card and do not crash the app.

## 5. Store / State Approach

Phase 3 uses React state in `App.tsx`:

- `appInfo`
- `appInfoStatus`
- `appInfoError`

The Tauri call is isolated in `studio/src/lib/tauri.ts`, and the DTO type lives in `studio/src/types/app.ts`. Zustand was not added because this phase only needs a single IPC DTO and simple loading/error state.

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
cargo fmt --manifest-path src-tauri/Cargo.toml
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
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 2 tests passed.
- `cargo test --manifest-path research-rs/Cargo.toml`: PASS, 152 tests passed.
- `git diff --check`: PASS.

No external OpenAI API, provider network, real run folder loading, or report data access was used.

## 8. Intentionally Not Implemented

Phase 3 intentionally does not implement:

- `list_runs`;
- `load_run_detail`;
- real run-folder reading;
- report artifact parsing;
- D3;
- PDF;
- regression matrix;
- provider calls;
- external AI calls;
- v5 core analysis changes.

## 9. Remaining Blockers

No blockers for Phase 3.

Known non-blocking note: npm reports 2 moderate dependency advisories. They were not force-fixed because that could introduce unrelated dependency churn.

## 10. Next Phase Recommendation

Phase 4 should define a read-only run-folder DTO contract before implementing `list_runs`. The frontend should continue to render typed DTOs only, while Rust/Tauri owns filesystem access.

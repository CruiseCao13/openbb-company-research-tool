# v6 Tauri Studio Phase 7 Artifact Opening Report

## 1. Files changed

- `src-tauri/src/lib.rs`
- `studio/src/components/RunDetailPanel.tsx`
- `studio/src/lib/tauri.ts`
- `studio/src/styles.css`
- `studio/src/types/app.ts`
- `reports/release_checks/v6_0/phase_07_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_07_artifact_opening_report.md`

Unrelated file left untouched:

- `reports/release_checks/v5_0/a_share_sector_smoke_report.md`

## 2. Commands implemented

Implemented two Tauri IPC commands:

- `open_artifact(path: String)`
- `reveal_in_folder(path: String)`

Both commands return:

```json
{
  "ok": true,
  "action": "open | reveal",
  "path": "...",
  "message": "..."
}
```

## 3. Path safety rules

The backend validates every requested artifact path before opening or revealing it.

Rules implemented:

- Empty paths are rejected.
- Path traversal components such as `..` are rejected.
- NUL bytes are rejected.
- Absolute paths outside `repo_root/reports` are rejected.
- Relative paths are resolved under `repo_root/reports`.
- Canonicalized paths must remain under the canonical reports root, which also blocks symlink escape when canonicalization is possible.
- `open_artifact` requires an existing file.
- `reveal_in_folder` accepts an existing file or directory under reports.
- Allowed opened files are limited to v5 run artifacts:
  - `report/*.md`
  - `report/*.pdf`
  - `dashboard.html`
  - `raw/provider_payload.json`
  - `metadata/*.json`
  - `audit/*.md`
  - `self_review/*.md`
  - `charts/*.png`
  - `pack/*.zip`
- The OS open/reveal wrapper uses `std::process::Command` with argument arrays, not shell string interpolation.
- The commands do not read file contents and do not mutate run folders.

## 4. Frontend artifact buttons

Added an Artifact Links card to the Run Detail UI.

Buttons:

- Open Markdown Report
- Open Dashboard
- Open PDF
- Open AI Usage
- Open Research Blueprint
- Open Validator Report
- Open Provider Payload
- Reveal Run Folder

Each button is disabled when the corresponding artifact path is missing. Actions go through the Tauri helper layer and never use direct filesystem links or `file://` navigation.

## 5. Error handling

Frontend behavior:

- Shows an action result message after open/reveal attempts.
- Shows a typed error message when Tauri returns an error.
- Shows a browser-preview warning if the Tauri runtime is unavailable.
- Does not pretend browser preview can open real local artifacts.

Backend behavior:

- Returns typed string errors for missing paths, unsupported artifacts, path traversal, outside-reports paths, and OS open/reveal failures.
- Unsupported platform behavior returns an error instead of crashing.

## 6. Tests added

Rust tests added:

- `open_artifact_rejects_path_traversal`
- `open_artifact_rejects_outside_reports`
- `open_artifact_rejects_missing_file`
- `open_artifact_accepts_report_markdown_under_reports`
- `reveal_in_folder_accepts_run_folder_under_reports`
- `reveal_in_folder_rejects_outside_reports`
- `artifact_commands_do_not_read_file_contents`

The tests use temporary fixture folders and validate the pure path-safety layer without invoking the host OS opener.

## 7. Validation commands run

- `npm run typecheck`
- `npm run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path research-rs/Cargo.toml`
- `git diff --check`

## 8. Validation results

- Frontend typecheck: PASS
- Frontend build: PASS
- Tauri cargo check: PASS
- Tauri cargo build: PASS
- Tauri tests: PASS, 23 passed
- Existing v5 Rust tests: PASS, 152 passed
- `git diff --check`: PASS

`npm install` reported two moderate npm audit advisories in the existing JavaScript dependency tree. They were not introduced as Phase 7 code changes and do not block the shell validation.

## 9. What is intentionally not implemented

- Full Markdown rendering
- Dashboard rendering inside the studio
- D3 charts
- PDF export
- Regression Matrix
- Artifact content preview
- Provider calls
- External OpenAI calls
- Any v5 report generation changes

## 10. Unrelated files left untouched

- `reports/release_checks/v5_0/a_share_sector_smoke_report.md`

## 11. Remaining blockers

None for Phase 7.

## 12. Final status

PASS

# v6 Tauri Studio Phase 9 Chart Grid Report

## 1. Files changed

- `src-tauri/src/lib.rs`
- `studio/src/components/RunDetailPanel.tsx`
- `studio/src/lib/tauri.ts`
- `studio/src/styles.css`
- `studio/src/types/app.ts`
- `reports/release_checks/v6_0/phase_09_preflight_git_check.md`
- `reports/release_checks/v6_0/phase_09_chart_grid_report.md`

## 2. Chart DTO changes

`RunDetail.charts` now includes richer chart metadata reconstructed from `charts/chart_manifest.json`:

```ts
{
  title: string;
  image_path: string | null;
  image_exists: boolean;
  source: string | null;
  status: string | null;
  why_selected: string | null;
  what_to_look_at: string | null;
  what_it_means: string | null;
  what_not_to_overread: string | null;
  next_check: string | null;
}
```

The backend sanitizes chart image paths and rejects absolute paths or traversal paths from the manifest.

## 3. ChartGrid UI behavior

Added a `ChartGrid` section inside the Run Detail UI after Audit Trail.

Each chart card shows:

- title
- status badge
- image preview when the PNG exists
- source
- what to look at
- what it means
- what not to overread
- next check
- Open Chart button using the existing `open_artifact` command

The grid is compact and responsive:

- three columns on wide layouts
- two columns on medium layouts
- one column on narrow layouts

## 4. Missing manifest behavior

If no charts are available from `charts/chart_manifest.json`, the UI shows:

```text
No chart manifest found for this run.
```

No chart generation is attempted.

## 5. Missing image behavior

If a manifest entry points to a missing image:

- `image_exists=false`
- the chart status falls back to `WARNING`
- the UI shows a warning tile instead of a broken image icon
- the Open Chart button is disabled

## 6. Chart limitation behavior

Every chart card displays a limitation line through one of:

- `what_not_to_overread`
- manifest limitation/warning fields mapped into the DTO
- fallback text:

```text
This chart is a visual aid only and does not create a buy/sell signal.
```

The UI does not imply buy/sell signals.

## 7. Artifact open integration

Chart image opening reuses the Phase 7 `open_artifact` IPC command. The frontend does not use direct filesystem links or `file://` navigation.

Image previews use Tauri's asset URL conversion for backend-returned artifact paths.

## 8. Tests added

Rust/Tauri tests added:

- `load_run_detail_reads_chart_manifest`
- `chart_grid_handles_missing_manifest`
- `chart_grid_marks_missing_image`
- `chart_metadata_includes_source_or_warning`
- `chart_artifact_path_stays_under_reports`

The tests use temporary fixture run folders and do not rely on real report data.

## 9. Validation commands run

- `npm install`
- `npm run typecheck`
- `npm run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path research-rs/Cargo.toml`
- `git diff --check`

## 10. Validation results

- Frontend typecheck: PASS
- Frontend build: PASS
- Tauri cargo check: PASS
- Tauri cargo build: PASS
- Tauri tests: PASS, 36 passed
- Existing v5 Rust tests: PASS, 152 passed
- `git diff --check`: PASS

`npm install` still reports two moderate npm audit advisories in the current JavaScript dependency tree. This was already present in earlier studio phases and was not introduced by Phase 9 chart rendering logic.

## 11. What is intentionally not implemented

- D3
- Sankey charts
- PDF export
- Regression Matrix
- chart generation
- chart manifest mutation
- full markdown report parsing
- provider calls
- external OpenAI calls
- v5 report renderer changes

## 12. Unrelated files left untouched

None.

## 13. Remaining blockers

None for Phase 9.

## 14. Final status

PASS

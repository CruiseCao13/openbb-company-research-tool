# v6 Tauri Research Studio Implementation Boundary

Status: phase 00 boundary

This document defines the implementation boundary for v6 Tauri Research Studio. v6 is a desktop browsing layer for existing v5 research artifacts. It is not a rewrite of the v5 research engine.

## Purpose

v6 Tauri Research Studio exists to help a user browse, inspect, compare, and package existing v5 run folders from a desktop UI.

Primary user jobs:

- Open an existing `reports/{TICKER}/runs/{RUN_ID}/` folder.
- Inspect report status, AI provenance, provider source, data gaps, quality checks, and generated artifacts.
- View Markdown report, dashboard, PDF status, charts, metadata, audits, and self-review files.
- Compare runs for the same ticker.
- Surface warnings clearly when a run used local fallback, provider public fallback, or data-limited analysis.

v6 must not claim that a report is more complete, more accurate, more external-AI-backed, or more provider-verified than the underlying v5 artifacts prove.

## Non-Goals

v6 must not modify v5 core analysis logic.

Forbidden in v6 phase 00 and phase 01 unless a later boundary update explicitly allows it:

- Changing provider fetching behavior.
- Changing AI prompts.
- Changing external AI gate behavior.
- Changing validators or report status rules.
- Changing training loops or eval sets.
- Changing report generation semantics.
- Creating new analysis artifacts that could be confused with v5 outputs.
- Calling external OpenAI or provider APIs from the Studio UI.

The Studio can request read-only summaries from Tauri commands. It cannot become another research pipeline.

## Architecture Boundary

The boundary is:

```text
v5 run folder artifacts
        |
        v
Rust/Tauri read-only filesystem commands
        |
        v
typed DTOs
        |
        v
React UI rendering
```

## Filesystem Access

Frontend code cannot directly read the filesystem.

Rules:

- React must not use Node filesystem APIs.
- React must not assume local absolute paths are readable.
- React must not scan `reports/` directly.
- React must not parse arbitrary local files outside typed DTOs returned by Tauri commands.
- Rust/Tauri owns all filesystem access.

Rust/Tauri command responsibilities:

- Resolve repo root or configured reports root.
- List allowed run folders.
- Read approved artifacts from a run folder.
- Validate that requested paths stay inside allowed report roots.
- Parse and normalize artifact files into DTOs.
- Return explicit missing/unavailable states instead of throwing UI-breaking errors.

## Network Access

Frontend code cannot call external network resources.

Rules:

- React must not call OpenAI, providers, GitHub, Eastmoney, yfinance, OpenBB, AKShare, Tushare, Baostock, or any remote endpoint.
- React must not send report content, metadata, provider payloads, or AI artifacts to external services.
- Tauri commands in phase 00 and phase 01 are read-only local commands by default.
- Any future network capability must be separately documented, user-visible, opt-in, and reviewed against secret safety and data privacy requirements.

## DTO Boundary

React only renders typed DTOs.

DTOs should be explicit and boring:

- `RunSummaryDto`
- `AiUsageDto`
- `ProviderStatusDto`
- `ReportStatusDto`
- `ArtifactAvailabilityDto`
- `DataGapDto`
- `QualitySummaryDto`
- `ChartManifestDto`
- `PackStatusDto`

DTO rules:

- Use booleans for facts such as `external_ai_used`, `local_mock_used`, `package_used`, and `mock`.
- Preserve source labels exactly, such as `external_openai`, `local_mock`, `cache`, `eastmoney_public`, and `akshare_package`.
- Represent missing artifacts explicitly.
- Include path display labels, but avoid leaking secrets or unnecessary local system paths into UI content.
- Never infer external AI usage from prose; use `metadata/ai_usage.json`.
- Never infer provider quality from ticker alone; use `raw/provider_payload.json` and `metadata/provider_status.json`.

## Allowed Directories

v6 implementation work is allowed in:

- `src-tauri/`
- `studio/`
- `research-rs/crates/research-studio/` if a shared Rust DTO/helper crate is needed
- `docs/v6*`
- `reports/release_checks/v6_0/`

These directories define the expected blast radius for v6 Studio work.

## Forbidden Directories

Do not modify these directories for v6 Studio work unless a future task explicitly requires it and explains why:

- `providers/`
- `research-rs/crates/research-ai/prompts/`
- `research-rs/crates/research-core/src/validation.rs`
- `research-rs/crates/research-batch/src/training.rs`
- `research-rs/crates/research-batch/src/training_case.rs`
- `training/`
- `training_cases/`
- `eval_sets/`
- v5 report generation logic in `research-rs/crates/research-report/`

If v6 discovers a core issue, file a release check or known limitation. Do not patch v5 core logic as part of Studio UI work.

## Read-Only Artifact Policy

v6 can read existing v5 outputs, including:

- `README.md`
- `report/*.md`
- `report/*.pdf`
- `dashboard.html`
- `raw/provider_payload.json`
- `metadata/*.json`
- `audit/*.md`
- `self_review/*.md`
- `charts/*`
- `pack/pack_manifest.json`

v6 must not rewrite these artifacts in place.

If v6 needs user notes or Studio-specific UI state, store them separately under a future Studio-owned path after that path is documented. Do not mix Studio state into v5 run folders during phase 00.

## Security Rules

- Never display a full API key.
- Never read `.env` in React.
- Never include `.env`, `reports/_cache`, provider caches, or temporary files in Studio packs.
- Redact strings matching real OpenAI key patterns before showing file previews.
- Treat every local report as user data.
- Do not send local report data to external services from the frontend.

## Testing Expectations

Phase 00 documentation checks:

- Boundary document exists.
- Release check exists.
- No v5 core code modified.

Phase 01 read-only Rust/Tauri command tests:

- Repo/report root resolution is deterministic.
- Path traversal outside the report root is rejected.
- Missing artifacts return typed unavailable states.
- `ai_usage.json` is parsed into DTOs without changing semantics.
- Provider source fields are preserved exactly.
- Local fallback warnings remain visible.
- No command requires `OPENAI_API_KEY`.
- No command calls external network by default.

Phase 02 UI tests:

- React renders DTO fixtures without filesystem access.
- AI source card reflects `ai_usage.json`.
- Provider source card reflects `provider_status.json` and `provider_payload.json`.
- Missing PDF/chart/report states are visible and honest.
- Local mock and public provider fallback warnings are prominent.

Phase 03 packaging tests:

- Desktop build does not include secrets.
- Studio bundle does not include generated cache.
- Pack/export actions cannot include `.env`.
- Smoke tests open a fixture run folder and render major panels.

## Phased Roadmap

Phase 00: Boundary and Design Contract

- Define implementation boundary.
- Define allowed and forbidden directories.
- Define DTO-first architecture.
- Do not write app code.

Phase 01: Read-Only Backend Skeleton

- Add Tauri/Rust read-only commands.
- Add DTO definitions and fixture-based tests.
- Support listing and opening existing v5 runs.
- No network calls.
- No v5 core logic changes.

Phase 02: Read-Only React Studio

- Render run list, report status, AI source, provider source, quality summary, data gaps, artifacts, charts, and pack status from DTOs.
- Keep UI honest about local fallback, cache, provider fallback, and missing artifacts.
- No direct filesystem access from React.

Phase 03: Packaging and User Workflow

- Build desktop package.
- Add safe open-folder workflow.
- Add fixture smoke tests and basic visual checks.
- Keep generated caches and secrets out of the bundle.

Phase 04: Optional Studio Enhancements

- Run comparison view.
- Artifact search within a selected run.
- Notes or annotations in a Studio-owned storage path.
- Any network or write capability requires a new boundary review before implementation.

## Release Rule

v6 Studio can be described as ready only when it remains a truthful browser of v5 artifacts. It must not hide data gaps, upgrade local fallback into external AI, relabel public provider fallback as package provider data, or mutate the research outputs it displays.

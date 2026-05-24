# v5 Error Handbook

This handbook explains common `research-rs` failures and the next command to run.

## Provider Failed

Meaning: every configured provider either timed out, returned empty data, or returned invalid JSON.

Next actions:

- Run `research-rs doctor`.
- Check `metadata/provider_status.json`.
- Check `audit/provider_validation.md`.
- Retry with `--force` only after confirming it was a transient provider issue.

## AI JSON Invalid

Meaning: the AI or local compact analyst output could not be parsed into the typed v5 schema.

Next actions:

- Check `ai/responses/`.
- Check `audit/schema_validation.md`.
- Rerun with the same provider data; cache should prevent redundant fetches.

## PDF Export Unavailable

Meaning: Markdown and HTML were generated, but the local PDF helper was unavailable or failed.

Next actions:

- Check `audit/pdf_export_report.md`.
- The Markdown report remains authoritative.
- Install optional PDF dependencies if a PDF artifact is required.

## Dashboard Missing

Meaning: `dashboard.html` was not written or failed visual lint.

Next actions:

- Check `audit/visual_lint_report.md`.
- Rerun `research-rs run TICKER --mode standard --force`.

## Chart Generation Failed

Meaning: chart helper execution failed or chart data was insufficient.

Next actions:

- Check `charts/Figure_01_data_gap.md`.
- Check `audit/chart_coverage_report.md`.
- Confirm `raw/provider_payload.json` has price and financial rows.

## Unsupported Claims

Meaning: report text made claims that the evidence map could not bind to locked data or a declared AI interpretation boundary.

Next actions:

- Check `metadata/evidence_map.json`.
- Check `audit/evidence_map.md`.
- Rerun with `--auto-fix --max-attempts 3`.

## Wrong Framework Conflict

Meaning: the company frame conflicts with provider profile, expected eval-set family, or known business clues.

Next actions:

- Check `metadata/company_understanding.json`.
- Check `metadata/research_blueprint.json`.
- Check generated training cases in `training_cases/corrections/`.

## Cache Stale

Meaning: outputs may have been generated from older provider or prompt inputs.

Next actions:

- Rerun with `--force` for one ticker.
- Compare `metadata/repro_manifest.json` cache keys and payload digests.

## A-Share Provider Field Missing

Meaning: the provider returned usable company data but not enough financial fields.

Next actions:

- Check `audit/parser_report.md`.
- Check `audit/normalizer_report.md`.
- Verify provider dependencies with `research-rs provider-health`.

## Debug a Run Folder

Start here:

1. `README.md`
2. `dashboard.html`
3. `metadata/report_status.json`
4. `metadata/repro_manifest.json`
5. `audit/validator_report.md`
6. `audit/visual_lint_report.md`
7. `audit/run_log.md`

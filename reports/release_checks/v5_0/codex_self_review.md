# Codex Self Review — v5.0

## What Was Built

- A new Rust workspace under `research-rs/` with core, AI, report, batch, and CLI crates.
- A Python provider bridge under `providers/` that writes a unified locked provider payload.
- JSON schemas for provider payload, company understanding, financial interpretation, research blueprint, AI self-review, report status, and batch summary.
- v5 single-company command that creates report, raw, metadata, audit, self-review, pack, and dashboard artifacts.
- v5 batch command that runs `broad_30_probe`, isolates ticker-level failures, writes matrix outputs, and generates local training cases.
- Per-run data inventory, data usage coverage, chart plan, evidence map, chart/table quality report, and PDF export audit artifacts.
- A Rust-side engineering control layer with error taxonomy types, provider status/cache metadata, compiler-style validation passes, table plans, run traces, batch traces, and pack manifest file digests.
- Formal parser/normalizer layer that writes typed normalized financials, normalized price history, parser report, and normalizer report before rendering.
- Rewrite status/trace, cache report, PDF status JSON, product quality score, and README review artifacts.

## What Was Not Built

- External paid AI calls are not enabled in this implementation.
- OpenBB direct normalization remains in the existing Python v4 workflow; the v5 provider bridge currently uses yfinance-compatible data for US/global tickers.
- AKShare/Tushare/Baostock are represented through the provider bridge but A-share normalization is still screening/fallback-only in this environment.

## Tests Run

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Python provider syntax checks
- `.venv/bin/python -m pytest -q`
- AAPL v5 run
- AAPL v5 Rust-brain run with `metadata/run_trace.json`, `metadata/validation_passes.json`, `metadata/table_plan.json`, and digest-aware pack manifest
- AAPL productization run with parser/normalizer reports, rewrite trace, cache report, PDF status JSON, and product quality score
- AAPL final productized run with `metadata/repro_manifest.json`, `audit/schema_validation.md`, `metadata/product_quality_score.json`, parser/normalizer outputs, dashboard, PDF, and pack
- 600519.SH final productized run with explicit A-share fallback / data-limited WARNING rather than fake full coverage
- `research-rs doctor` and `research-rs samples`
- smoke_12 three-name batch probe with `batch_trace.json` and runtime matrix
- 600519.SH v5 run
- broad_30 v5 batch
- broad_30 content quality run
- broad_500 mixed US/CN quality segment with `--limit 50 --offset 225`

## Batch Result

Latest validation batch:

`reports/batch_runs/v5_broad30_productized_final3`

Summary:

- Total tickers: 5
- PASS: 5
- WARNING: 0
- FAIL: 0
- External AI calls: 0

This was a capped broad_30 segment for the final productization pass, not a claim that full broad_30 or broad_500 completed in this turn.

## AI Credit Usage

External paid AI calls: 0.

The current engine uses local compact analyst fallback. Full reports, CSVs, and
charts were not sent to any external AI service.

## Content Quality Review

Latest quality runs:

- `reports/quality_runs/v5_quality_broad_30`
- `reports/quality_runs/v5_quality_broad_500_mixed_50`
- `reports/quality_runs/v5_quality_productized_final3`

The broad_30 quality run scored 30 reports with average quality 84.4, zero
FAIL grades, and zero hard failures. The mixed broad_500 segment scored 50
US/CN reports with average quality 81.0, zero FAIL grades, and explicit
human-review caps for fallback/data-limited cases.
The final productization quality smoke scored 5 reports with average quality
86.0, zero FAIL grades, and zero hard failures.

## Provider Limitations

- US/global runs use yfinance-compatible provider bridge behavior.
- 600519.SH generated a clear A-share screening/fallback run because the full
  AKShare/Tushare/Baostock normalization adapter is not complete here.
- `research-rs doctor` confirms AKShare is not installed and no Tushare token is configured in this environment.

## Display Review

See `reports/release_checks/v5_0/display_review.md`.

## Display and Formatting Review

- Charts generated: Yes. AAPL generated five PNG charts plus chart manifest.
- Figure numbering: Yes, `Figure_01` through `Figure_05`.
- Chart explanations: Yes, each figure section includes what to look at, what it means, what not to overread, and next check.
- Tables too wide: No known wide report tables in v5 output.
- Units: `metadata/unit_policy.json` is generated for each run.
- Dashboard openable: Yes, static HTML is generated for single runs and batch.
- PDF export: Yes, a dependency-free text-first PDF is generated for Markdown reports. It preserves chart explanations and source notes but does not embed PNG chart images.
- Data usage coverage: Yes, `metadata/data_usage_coverage.json` and `audit/data_usage_coverage_report.md` map fetched critical fields to report/chart/table/appendix destinations or data gaps.
- Chart/table judge: Yes, `metadata/chart_table_quality.json` and `audit/chart_table_quality_report.md` score relevance, readability, explanation, units, source trace, and visual polish.
- Evidence map: Yes, key claims are mapped to provider sections, chart references, table references, confidence, and unsupported-claim status.
- Table plan: Yes, `metadata/table_plan.json` and `audit/table_selection_report.md` are generated before report rendering.
- Parser/normalizer: Yes, `data/normalized_financials.json`, `data/normalized_price_history.json`, `audit/parser_report.md`, and `audit/normalizer_report.md` are generated before report rendering.
- Validation passes: Yes, `metadata/validation_passes.json` records compiler-style provider, AI schema, money flow, evidence, chart/table, visual, and PDF passes.
- Rewrite trace: Yes, `metadata/rewrite_status.json` and `audit/rewrite_trace.md` record whether operational rewrite was needed.
- Cache report: Yes, `metadata/cache_summary.json` and `audit/cache_report.md` record current cache hits and explicit limitations.
- Traceability: Yes, single runs write `metadata/run_trace.json` and `audit/run_log.md`; batch runs write `batch_trace.json`.
- Reproducibility: Yes, single runs write `metadata/repro_manifest.json` with git commit, provider digest, prompt versions, schema versions, renderer versions, command, and environment summary.
- Schema validation: Yes, runs write `audit/schema_validation.md`.
- Provider health: Yes, `reports/release_checks/provider_health.md` is generated by `research-rs doctor`.
- Sample gallery: Yes, `reports/samples/index.html` and `reports/samples/README.md` are generated by `research-rs samples`.
- Pack manifest: Yes, pack manifests include PDF/dashboard/chart flags plus per-file size and SHA-256 digest.
- Folder structure: Clear v5 run folders with report/raw/metadata/audit/self_review/data/charts/pack/dashboard.
- Template flavor: Reduced but not eliminated; local compact analyst remains less natural than future external AI.
- Unsupported claims: Flagged in AI self-review instead of silently presented.
- Raw NaN/null: Not present in report surface in the validation run.

## Efficiency Review

- AI calls total: 0 external paid calls.
- AI calls avoided by cache: External AI is disabled; local compact fallback used.
- Provider cache hits: Provider cache is path-based and avoids refetch unless `--force` is used.
- Stage timing: `run_trace.json` records provider fetch, validation, local analysis, report render, and pack durations.
- Chart cache hits: Chart generation is deterministic; deeper digest-based chart cache remains future work.
- Batch runtime: broad_30 validation completed in roughly one minute in this environment.
- Slowest stage: provider fetch / yfinance response.
- Repeated work found: yes, chart/provider digest skip tracking is still shallow.
- Unnecessary generated files: release commit excludes cache and `research-rs/target`.

## Honesty Review

- Real provider calls: US/global yfinance-compatible bridge was called.
- Provider fallback: A-share `600519.SH` degraded to screening/fallback behavior because full AKShare/Tushare/Baostock normalization is not complete.
- Mock outputs: No fake successful provider data was created for A-share; fallback is explicit.
- External AI called: No.
- Local fallback boundary: The local compact analyst is deterministic and useful for regression, but it is not equivalent to full AI research judgment.
- Secret scan: no `OPENAI_API_KEY`, `TUSHARE_TOKEN`, or long `sk-...` secrets were found outside ignored/generated dependency paths.

## Remaining Risks

- Local compact analyst is deterministic and useful for regression, but it is
  not equivalent to a full external analyst model.
- Batch guardrails use eval-set expected families to prevent bad training cases
  during validation. Production single-company runs do not inject those labels.
- More sector-specific validators are needed before broad_200 should be treated
  as product-grade.
- broad_500 is staged and supported, but only a 50-name mixed segment was run
  in this pass. Full broad_500 remains a later pressure test.

## Next Recommended Work

1. Add schema-validated external AI client with cache and strict budget controls.
2. Finish A-share provider normalization.
3. Add numeric claim tracing against locked provider payload.
4. Expand v5 batch summaries with richer failure taxonomy.

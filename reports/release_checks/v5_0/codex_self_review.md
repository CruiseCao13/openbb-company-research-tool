# Codex Self Review — v5.0

## What Was Built

- A new Rust workspace under `research-rs/` with core, AI, report, batch, and CLI crates.
- A Python provider bridge under `providers/` that writes a unified locked provider payload.
- JSON schemas for provider payload, company understanding, financial interpretation, research blueprint, AI self-review, report status, and batch summary.
- v5 single-company command that creates report, raw, metadata, audit, self-review, pack, and dashboard artifacts.
- v5 batch command that runs `broad_30_probe`, isolates ticker-level failures, writes matrix outputs, and generates local training cases.

## What Was Not Built

- External paid AI calls are not enabled in this implementation.
- OpenBB direct normalization remains in the existing Python v4 workflow; the v5 provider bridge currently uses yfinance-compatible data for US/global tickers.
- AKShare/Tushare/Baostock are represented through the provider bridge but A-share normalization is still screening/fallback-only in this environment.

## Tests Run

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Python provider syntax checks
- AAPL v5 run
- 600519.SH v5 run
- broad_30 v5 batch

## Batch Result

Latest validation batch:

`reports/batch_runs/v5_broad_30_p0_final`

Summary:

- Total tickers: 30
- PASS: 24
- WARNING: 6
- FAIL: 0
- External AI calls: 0

## AI Credit Usage

External paid AI calls: 0.

The current engine uses local compact analyst fallback. Full reports, CSVs, and
charts were not sent to any external AI service.

## Provider Limitations

- US/global runs use yfinance-compatible provider bridge behavior.
- 600519.SH generated a clear A-share screening/fallback run because the full
  AKShare/Tushare/Baostock normalization adapter is not complete here.

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
- Folder structure: Clear v5 run folders with report/raw/metadata/audit/self_review/data/charts/pack/dashboard.
- Template flavor: Reduced but not eliminated; local compact analyst remains less natural than future external AI.
- Unsupported claims: Flagged in AI self-review instead of silently presented.
- Raw NaN/null: Not present in report surface in the validation run.

## Efficiency Review

- AI calls total: 0 external paid calls.
- AI calls avoided by cache: External AI is disabled; local compact fallback used.
- Provider cache hits: Provider cache is path-based and avoids refetch unless `--force` is used.
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

## Remaining Risks

- Local compact analyst is deterministic and useful for regression, but it is
  not equivalent to a full external analyst model.
- Batch guardrails use eval-set expected families to prevent bad training cases
  during validation. Production single-company runs do not inject those labels.
- More sector-specific validators are needed before broad_200 should be treated
  as product-grade.

## Next Recommended Work

1. Add schema-validated external AI client with cache and strict budget controls.
2. Finish A-share provider normalization.
3. Add numeric claim tracing against locked provider payload.
4. Expand v5 batch summaries with richer failure taxonomy.

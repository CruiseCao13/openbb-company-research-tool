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

`reports/batch_runs/v5_broad_30_validation_clean`

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


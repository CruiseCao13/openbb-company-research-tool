# v5 Core Data Accuracy Pass 01
Date: 2026-05-25
Scope: core data accuracy, field mapping, unit policy, evidence map, money-flow data source checks, chart data coverage, and report-status honesty. No external AI, no v6 UI work, no broad_500 external run.
Final status: WARNING
## Companies Checked
| Ticker | Run | Provider | Source | Market | Currency | Mock | Report Status | Missing Required Artifacts | New Accuracy Audits Present? |
|---|---|---|---|---|---|---:|---|---|---|
| AAPL | `reports/AAPL/runs/ci_broad_500_dryrun_10_batch_AAPL` | yfinance | yfinance | US | USD | None | WARNING | data/valuation_snapshot.json | No, existing run predates pass |
| RKLB | `reports/RKLB/runs/external_regression_pass_01_batch_RKLB` | yfinance | yfinance | US | USD | None | WARNING | data/valuation_snapshot.json | No, existing run predates pass |
| GOOGL | `reports/GOOGL/runs/external_regression_pass_01_batch_GOOGL` | yfinance | yfinance | US | USD | None | WARNING | data/valuation_snapshot.json | No, existing run predates pass |
| CAT | `reports/CAT/runs/external_regression_pass_01_batch_CAT` | yfinance | yfinance | US | USD | None | WARNING | data/valuation_snapshot.json | No, existing run predates pass |
| ISRG | `reports/ISRG/runs/external_regression_pass_01_batch_ISRG` | yfinance | yfinance | US | USD | None | WARNING | data/valuation_snapshot.json | No, existing run predates pass |
| JPM | `reports/JPM/runs/external_regression_pass_01_batch_JPM` | yfinance | yfinance | US | USD | None | WARNING | data/valuation_snapshot.json | No, existing run predates pass |
| 600519.SH | `reports/600519.SH/runs/a_share_sector_frame_guard_600519_SH` | eastmoney_public | Eastmoney public endpoint | CN_A | CNY | False | PASS | data/valuation_snapshot.json | No, existing run predates pass |
| 000001.SZ | `reports/000001.SZ/runs/a_share_sector_frame_guard_000001_SZ` | eastmoney_public | Eastmoney public endpoint | CN_A | CNY | False | PASS | data/valuation_snapshot.json | No, existing run predates pass |
| 300750.SZ | `reports/300750.SZ/runs/a_share_sector_frame_guard_300750_SZ` | eastmoney_public | Eastmoney public endpoint | CN_A | CNY | False | PASS | data/valuation_snapshot.json | No, existing run predates pass |
| 601318.SH | `reports/601318.SH/runs/a_share_sector_frame_guard_601318_SH` | eastmoney_public | Eastmoney public endpoint | CN_A | CNY | False | PASS | data/valuation_snapshot.json | No, existing run predates pass |
| 600276.SH | `reports/600276.SH/runs/a_share_sector_frame_guard_600276_SH` | eastmoney_public | Eastmoney public endpoint | CN_A | CNY | False | PASS | data/valuation_snapshot.json | No, existing run predates pass |
| 601899.SH | `reports/601899.SH/runs/a_share_sector_frame_guard_601899_SH` | eastmoney_public | Eastmoney public endpoint | CN_A | CNY | False | PASS | data/valuation_snapshot.json | No, existing run predates pass |

## Provider Sources
- US sample runs use `yfinance` / `yfinance` source with USD market data.
- A-share sample runs use `eastmoney_public` with `Eastmoney public endpoint`, `package_used=false`, `mock=false`, `market=CN_A`, and `currency=CNY`.

## Data Coverage Summary
- Existing latest runs contain raw provider payload, normalized financials, data inventory, unit policy, evidence map, money-flow map, chart plan, chart/table quality metadata, chart manifest, data quality audit, provider validation audit, and validator report.
- Existing latest runs do not yet contain standalone `data/valuation_snapshot.json` because they predate this pass. The normalizer now writes this artifact for future runs.
- Existing latest runs do not yet contain the four new accuracy audit markdown files because they predate this pass. The renderer now writes them for future runs.

## Numeric Claim Accuracy
- Added `audit/evidence_numeric_accuracy_report.md` generation. It counts report numeric-looking claims and requires locked numeric evidence plus `metadata/evidence_map.json`.
- Evidence map now includes `locked_data_supported` entries for numeric statement rows from income statement, balance sheet, and cash flow.
- Remaining limitation: this first pass checks presence and traceability surfaces; exact phrase-level numeric extraction is conservative and should be deepened in a later pass.

## Money Flow Accuracy
- Added `audit/money_flow_accuracy_report.md` generation. It checks OCF, capex, FCF consistency when all values exist.
- If FCF differs materially from `operating cash flow + capex`, report status is forced to FAIL.
- Generic money-flow language is marked WARNING. Bank/insurance industrial FCF framing is a FAIL.

## Chart Data Accuracy
- Added `audit/chart_data_accuracy_report.md` generation. It verifies chart manifest parseability, source disclosure, generated artifact existence, and data-gap/PASS consistency.
- Fixed chart provider field matching for A-share Chinese revenue and cash-flow metrics such as `营业收入`, `经营现金流`, and `购建固定资产`.

## Unit Policy Accuracy
- Added `audit/unit_policy_accuracy_report.md` generation. US market expects USD; CN_A expects CNY/RMB and `mock=false`.
- Unit-policy failures force report status to FAIL; warnings force report status to WARNING if it would otherwise pass.

## A-share Accuracy
- Checked 600519.SH, 000001.SZ, 300750.SZ, 601318.SH, 600276.SH, 601899.SH existing latest runs.
- All checked A-share latest runs disclose `eastmoney_public`, source `Eastmoney public endpoint`, `package_used=false`, `mock=false`, `market=CN_A`, `currency=CNY`.
- Existing A-share latest runs are WARNING for this pass because they predate standalone valuation snapshot and new accuracy audit artifacts.

## US Accuracy
- Checked AAPL, RKLB, GOOGL, CAT, ISRG, JPM existing latest runs.
- All checked US latest runs disclose `market=US` and `currency=USD`.
- Existing US latest runs are WARNING for this pass because they predate standalone valuation snapshot and new accuracy audit artifacts.

## Hard Failures
- None found in the static latest-run metadata checks.

## Warnings
- AAPL: existing latest run missing data/valuation_snapshot.json
- AAPL: latest existing run predates new accuracy audits
- RKLB: existing latest run missing data/valuation_snapshot.json
- RKLB: latest existing run predates new accuracy audits
- GOOGL: existing latest run missing data/valuation_snapshot.json
- GOOGL: latest existing run predates new accuracy audits
- CAT: existing latest run missing data/valuation_snapshot.json
- CAT: latest existing run predates new accuracy audits
- ISRG: existing latest run missing data/valuation_snapshot.json
- ISRG: latest existing run predates new accuracy audits
- JPM: existing latest run missing data/valuation_snapshot.json
- JPM: latest existing run predates new accuracy audits
- 600519.SH: existing latest run missing data/valuation_snapshot.json
- 600519.SH: latest existing run predates new accuracy audits
- 000001.SZ: existing latest run missing data/valuation_snapshot.json
- 000001.SZ: latest existing run predates new accuracy audits
- 300750.SZ: existing latest run missing data/valuation_snapshot.json
- 300750.SZ: latest existing run predates new accuracy audits
- 601318.SH: existing latest run missing data/valuation_snapshot.json
- 601318.SH: latest existing run predates new accuracy audits
- 600276.SH: existing latest run missing data/valuation_snapshot.json
- 600276.SH: latest existing run predates new accuracy audits
- 601899.SH: existing latest run missing data/valuation_snapshot.json
- 601899.SH: latest existing run predates new accuracy audits

## Files Changed
- `research-rs/crates/research-core/src/normalizer.rs`: writes standalone `data/valuation_snapshot.json`.
- `research-rs/crates/research-report/src/renderer.rs`: writes numeric evidence, money-flow, chart-data, and unit-policy accuracy audits; hard accuracy failures downgrade report status.
- `providers/chart_provider.py`: recognizes Chinese A-share revenue and cash-flow metric names.
- `research-rs/crates/research-core/src/tests.rs`: adds cash-flow sign, FCF, and currency tests.
- `research-rs/crates/research-report/src/tests.rs`: adds evidence, bank/insurance FCF, chart source, unit policy, and money-flow conflict tests.

## Tests Run
- `cargo test --manifest-path research-rs/Cargo.toml` - PASS during implementation smoke.
- Full required validation is run after this report before commit.

## Remaining Blockers
- Existing checked run folders predate this pass; rerunning selected samples locally will be needed to populate the new accuracy audit files and standalone valuation snapshot in those run folders. These generated run outputs are intentionally not committed in this pass.

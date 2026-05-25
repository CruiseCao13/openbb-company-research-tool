# v5 Core Data Accuracy Rerun Pass 01

Date: 2026-05-25

Scope: reran 12 representative v5 samples with `--ai local --provider auto --force` to verify core data accuracy artifacts, unit policy, evidence map, Money Flow, chart data coverage, and report-status honesty. No external OpenAI API was used.

Final status: WARNING

## Companies Rerun

| Ticker | Run path | Provider source | Market | Currency | Frame | Report status |
|---|---|---|---|---|---|---|
| AAPL | `reports/AAPL/runs/core_data_accuracy_rerun_01_aapl` | yfinance / yfinance | US | USD | Mature Consumer Technology Compounder | WARNING |
| RKLB | `reports/RKLB/runs/core_data_accuracy_rerun_01_rklb` | yfinance / yfinance | US | USD | Speculative Aerospace / Space Systems | WARNING |
| GOOGL | `reports/GOOGL/runs/core_data_accuracy_rerun_01_googl` | yfinance / yfinance | US | USD | Platform Internet / Digital Ads / Cloud | WARNING |
| CAT | `reports/CAT/runs/core_data_accuracy_rerun_01_cat` | yfinance / yfinance | US | USD | Cyclical / Industrial Machinery | WARNING |
| ISRG | `reports/ISRG/runs/core_data_accuracy_rerun_01_isrg` | yfinance / yfinance | US | USD | Medical Devices / Surgical Robotics | WARNING |
| JPM | `reports/JPM/runs/core_data_accuracy_rerun_01_jpm` | yfinance / yfinance | US | USD | Financials / Bank-like Screening | WARNING |
| 600519.SH | `reports/600519.SH/runs/core_data_accuracy_rerun_01_600519_sh` | eastmoney_public / Eastmoney public endpoint | CN_A | CNY | A-share Premium Baijiu / Consumer Brand | PASS |
| 000001.SZ | `reports/000001.SZ/runs/core_data_accuracy_rerun_01_000001_sz` | eastmoney_public / Eastmoney public endpoint | CN_A | CNY | Financials / Bank-like Screening | PASS |
| 300750.SZ | `reports/300750.SZ/runs/core_data_accuracy_rerun_01_300750_sz` | eastmoney_public / Eastmoney public endpoint | CN_A | CNY | New Energy / Battery Manufacturing | PASS |
| 601318.SH | `reports/601318.SH/runs/core_data_accuracy_rerun_01_601318_sh` | eastmoney_public / Eastmoney public endpoint | CN_A | CNY | Insurance / Integrated Financials | PASS |
| 600276.SH | `reports/600276.SH/runs/core_data_accuracy_rerun_01_600276_sh` | eastmoney_public / Eastmoney public endpoint | CN_A | CNY | Pharma / Innovative Drug Portfolio | PASS |
| 601899.SH | `reports/601899.SH/runs/core_data_accuracy_rerun_01_601899_sh` | eastmoney_public / Eastmoney public endpoint | CN_A | CNY | Mining / Nonferrous Metals / Commodity Cycle | PASS |

## Audit Artifact Presence

| Ticker | Required artifacts | Valuation snapshot | Numeric evidence | Money flow | Chart data | Unit policy | Chart manifest |
|---|---|---|---|---|---|---|---|
| AAPL | PASS | PASS | PASS | WARNING | PASS | PASS | PASS |
| RKLB | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| GOOGL | PASS | PASS | PASS | WARNING | PASS | PASS | PASS |
| CAT | PASS | PASS | PASS | WARNING | PASS | PASS | PASS |
| ISRG | PASS | PASS | PASS | WARNING | PASS | PASS | PASS |
| JPM | PASS | PASS | PASS | WARNING | PASS | PASS | PASS |
| 600519.SH | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| 000001.SZ | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| 300750.SZ | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| 601318.SH | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| 600276.SH | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| 601899.SH | PASS | PASS | PASS | PASS | PASS | PASS | PASS |

## Provider / Market / Currency Result

- US samples: AAPL, RKLB, GOOGL, CAT, ISRG, JPM all reran with `market=US`, `currency=USD`, provider `yfinance`, `mock=false`.
- CN_A samples: 600519.SH, 000001.SZ, 300750.SZ, 601318.SH, 600276.SH, 601899.SH all reran with `market=CN_A`, `currency=CNY`, provider `eastmoney_public`, adapter `akshare_compatible_fallback`, `package_used=false`, `mock=false`.
- A-share fallback is a disclosed public provider fallback, not mock data.

## Numeric Evidence Result

- 12/12 generated `audit/evidence_numeric_accuracy_report.md`.
- 12/12 numeric evidence audits are PASS.
- No report-status PASS was found with an evidence numeric hard failure.

## Money Flow Result

- 12/12 generated `audit/money_flow_accuracy_report.md`.
- 7/12 money-flow audits are PASS.
- 5/12 US money-flow audits are WARNING because local fallback wording is still too generic, not because of a locked-data contradiction.
- No FCF sign/calculation hard failure was detected.
- Bank/insurance industrial FCF misuse was not detected after the CAT/JPM frame guard repair.

## Chart Accuracy Result

- 12/12 generated `audit/chart_data_accuracy_report.md`.
- 12/12 chart data audits are PASS.
- 12/12 have `charts/chart_manifest.json`.
- Chart provider now recognizes Chinese A-share revenue and cash-flow metric names.

## Unit Policy Result

- 12/12 generated `audit/unit_policy_accuracy_report.md`.
- 12/12 unit policy audits are PASS.
- No USD/CNY market-currency mismatch detected.

## Report Status Summary

- AAPL: `WARNING`, human_review_required=True, external_ai_used=False, local_mock_used=True
- RKLB: `WARNING`, human_review_required=True, external_ai_used=False, local_mock_used=True
- GOOGL: `WARNING`, human_review_required=True, external_ai_used=False, local_mock_used=True
- CAT: `WARNING`, human_review_required=True, external_ai_used=False, local_mock_used=True
- ISRG: `WARNING`, human_review_required=True, external_ai_used=False, local_mock_used=True
- JPM: `WARNING`, human_review_required=True, external_ai_used=False, local_mock_used=True
- 600519.SH: `PASS`, human_review_required=False, external_ai_used=False, local_mock_used=True
- 000001.SZ: `PASS`, human_review_required=False, external_ai_used=False, local_mock_used=True
- 300750.SZ: `PASS`, human_review_required=False, external_ai_used=False, local_mock_used=True
- 601318.SH: `PASS`, human_review_required=False, external_ai_used=False, local_mock_used=True
- 600276.SH: `PASS`, human_review_required=False, external_ai_used=False, local_mock_used=True
- 601899.SH: `PASS`, human_review_required=False, external_ai_used=False, local_mock_used=True

## Hard Failures

- None.

## Warnings

- AAPL: audit/money_flow_accuracy_report.md status WARNING
- AAPL: report_status WARNING because local AI/human review or money-flow specificity
- RKLB: report_status WARNING because local AI/human review or money-flow specificity
- GOOGL: audit/money_flow_accuracy_report.md status WARNING
- GOOGL: report_status WARNING because local AI/human review or money-flow specificity
- CAT: audit/money_flow_accuracy_report.md status WARNING
- CAT: report_status WARNING because local AI/human review or money-flow specificity
- ISRG: audit/money_flow_accuracy_report.md status WARNING
- ISRG: report_status WARNING because local AI/human review or money-flow specificity
- JPM: audit/money_flow_accuracy_report.md status WARNING
- JPM: report_status WARNING because local AI/human review or money-flow specificity
- 600519.SH: A-share Eastmoney public fallback disclosed
- 000001.SZ: A-share Eastmoney public fallback disclosed
- 300750.SZ: A-share Eastmoney public fallback disclosed
- 601318.SH: A-share Eastmoney public fallback disclosed
- 600276.SH: A-share Eastmoney public fallback disclosed
- 601899.SH: A-share Eastmoney public fallback disclosed

## Fixes Made

- Added deterministic local frame guards for CAT and JPM after rerun exposed wrong local frames.
- Added regression tests for CAT as industrial machinery and JPM as bank-like screening.

## Files Changed

- `research-rs/crates/research-ai/src/company_understanding.rs`
- `research-rs/crates/research-ai/src/tests.rs`
- `reports/release_checks/v5_0/core_data_accuracy_rerun_pass_01.md`

## Notes On Generated Runs

The 12 rerun folders were generated locally as validation artifacts and are intentionally not staged for commit. The release-check report records their paths and audit statuses.

## Final Status Rule

- PASS requires all 12 required artifacts and zero hard failures.
- WARNING is used because optional quality warnings remain: local AI report status on US samples, generic local Money Flow wording on five US samples, and disclosed Eastmoney public fallback for A-share samples.

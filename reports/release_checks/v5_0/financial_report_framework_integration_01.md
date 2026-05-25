# v5 Financial Report Reading Framework Integration 01

Date: 2026-05-26

Final status: WARNING

This pass integrates the user's financial report reading framework into v5 as human-readable methodology docs, a machine-readable YAML rubric, per-run framework coverage artifacts, and a short report summary section.

No external OpenAI API was used. No v6 UI, provider rewrite, D3, PDF styling, broad_500, or training-scale work was performed.

## Docs Added

- `docs/investment_rubric/financial_report_reading_framework.zh.md`
- `docs/investment_rubric/financial_report_reading_framework.en.md`
- `docs/investment_rubric/financial_report_reading_framework.yaml`

The Chinese document is Chinese-only. The English document is English-only. Both state that the framework is for understanding companies and is not investment advice.

## YAML Section Count

YAML file: `docs/investment_rubric/financial_report_reading_framework.yaml`

Section count: 11

Sections:

1. `business_model`
2. `revenue_growth`
3. `gross_margin`
4. `operating_profit`
5. `net_profit`
6. `cash_flow`
7. `balance_sheet`
8. `key_business_metrics`
9. `guidance`
10. `market_expectations`
11. `valuation`

## Artifacts Updated

Existing company-specific artifacts now include `framework_sections` references:

- `metadata/company_fact_sheet.json` -> `business_model`
- `metadata/revenue_engine_map.json` -> `business_model`, `revenue_growth`
- `metadata/cost_structure_map.json` -> `gross_margin`, `operating_profit`
- `metadata/capital_allocation_map.json` -> `net_profit`, `balance_sheet`
- `metadata/money_flow_mechanism.json` -> `cash_flow`
- `metadata/company_specific_questions.json` -> `key_business_metrics`, `guidance`, `market_expectations`, `valuation`

New per-run artifacts:

- `metadata/financial_report_framework_coverage.json`
- `audit/financial_report_framework_coverage.md`

The English report now includes:

- `Financial Report Framework Coverage`

The Chinese report mode includes:

- `财报阅读框架覆盖情况`

## Samples Rerun

All samples were rerun with:

- `--ai local`
- `--provider auto`
- `--force`
- no external OpenAI API

| Ticker | Run ID | Report status | Coverage JSON | Coverage audit | Sections | Guidance | Market expectations | Valuation | Report summary |
|---|---|---|---|---|---:|---|---|---|---|
| AAPL | `financial_report_framework_01_aapl` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| GOOGL | `financial_report_framework_01_googl` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| CAT | `financial_report_framework_01_cat` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| ISRG | `financial_report_framework_01_isrg` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| JPM | `financial_report_framework_01_jpm` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| RKLB | `financial_report_framework_01_rklb` | WARNING | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| 600519.SH | `financial_report_framework_01_600519_sh` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| 000001.SZ | `financial_report_framework_01_000001_sz` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| 300750.SZ | `financial_report_framework_01_300750_sz` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| 601318.SH | `financial_report_framework_01_601318_sh` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| 600276.SH | `financial_report_framework_01_600276_sh` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |
| 601899.SH | `financial_report_framework_01_601899_sh` | PASS | PASS | PASS | 11 | DATA_GAP | DATA_GAP | WARNING | PASS |

RKLB remains report-level WARNING because local fallback is data-limited for speculative aerospace. Framework coverage artifacts still generated correctly.

## Coverage Summary

Across 12 samples and 132 section checks:

- PASS: 17
- WARNING: 67
- DATA_GAP: 48
- FAIL: 0

No unsupported valuation or market-expectation conclusion was detected.

## Missing Sections By Sample

Guidance:

- DATA_GAP for 12/12 samples.
- Reason: provider payload does not provide explicit revenue guidance, margin guidance, full-year outlook, or management commentary.

Market expectations:

- DATA_GAP for 12/12 samples.
- Reason: provider payload does not provide actual-vs-expectation, beat/miss, or expectation revision data.

Key business metrics:

- DATA_GAP for 12/12 samples.
- Reason: sector KPI coverage is not consistently present in the compact provider payload.

Valuation:

- WARNING for 12/12 samples.
- Reason: valuation method fit can be discussed, but implied growth, margin of safety, and downside-if-growth-misses are not fully supported by locked data.

## Guidance / Market Expectation Data Gap Summary

Guidance and market expectations are explicitly marked DATA_GAP unless provider data supports them. The report summary and audit file state that these areas cannot be concluded from price action or generic market narrative.

## Valuation Safety Result

Valuation coverage is limited to:

- valuation method fit
- missing inputs
- implied expectation checks where data exists

It does not generate:

- investment recommendation
- target price
- unsupported market expectation claim

Result: PASS

## Warnings

- Guidance data is absent for all 12 samples.
- Market expectation data is absent for all 12 samples.
- Sector-specific KPI coverage remains partial / DATA_GAP.
- Valuation is WARNING rather than PASS because the framework requires implied growth and downside checks that compact provider data does not fully support.

## Hard Failures

Hard failures: 0

No section generated an unsupported valuation or market-expectation conclusion.

## Validation Commands

Focused validation during implementation:

- `cargo test --manifest-path research-rs/Cargo.toml -p research-report`

Full validation is recorded in the final response.

## Final Assessment

The framework is integrated and auditable. The final status is WARNING because guidance, market expectations, sector KPIs, and full valuation expectation analysis are correctly disclosed as data gaps instead of being overclaimed.

Final status: WARNING

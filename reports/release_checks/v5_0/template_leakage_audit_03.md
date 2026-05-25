# v5 Template Leakage Audit 03

Date: 2026-05-25

Baseline reports:

- `reports/release_checks/v5_0/template_leakage_audit_01.md`
- `reports/release_checks/v5_0/template_leakage_audit_02.md`

Final status: PASS for chart leakage, WARNING for broader non-chart report scaffolding.

## Audit 01 Summary

Audit 01 found high-severity local fallback leakage:

- frame-label company identity
- frame-based business-model prose
- generic money-flow paragraphs
- repeated chart explanations

Product-quality status was WARNING.

## Audit 02 Summary

Audit 02 confirmed that company-specific analysis artifacts reduced the worst business-model and money-flow leakage:

- `company_fact_sheet.json`
- `revenue_engine_map.json`
- `cost_structure_map.json`
- `capital_allocation_map.json`
- `money_flow_mechanism.json`
- `company_specific_questions.json`

Remaining blocker:

- chart explanation boilerplate still repeated across companies

Product-quality status remained WARNING.

## Audit 03 Scope

Audit 03 checks the 12 `chart_observation_map_01_*` reruns:

- AAPL
- GOOGL
- CAT
- ISRG
- JPM
- RKLB
- 600519.SH
- 000001.SZ
- 300750.SZ
- 601318.SH
- 600276.SH
- 601899.SH

Mode:

- local AI fallback
- provider auto
- no external OpenAI API

## Chart Leakage Improvements

New artifacts:

- `metadata/chart_observation_map.json`
- `metadata/chart_claim_map.json`
- `audit/chart_observation_quality_report.md`

Results:

- chart observation maps generated: 12/12
- chart claim maps generated: 12/12
- chart observation quality PASS: 12/12
- old generic chart phrase hits: 0

Old repeated chart phrases removed:

- `Compare the company price path with the benchmark`
- `This can show whether the stock has created relative price`
- `A price chart cannot prove the stock is cheap`
- `The useful question is whether growth is converting`
- `A cash-flow bridge cannot prove future runway`

## What Replaced Generic Chart Text

Each chart block now includes:

- company-specific observation
- ticker
- research frame
- source fields
- latest locked observation
- what the chart can support
- what it cannot prove
- next check

Example behavior:

- AAPL chart text connects observations to the mature consumer technology / hardware-services frame.
- GOOGL chart text connects observations to platform internet / digital ads / cloud.
- JPM chart text avoids industrial FCF logic and points to bank economics.
- RKLB chart text connects to speculative aerospace, cash burn, financing, and data gaps.
- CN_A chart text connects to each A-share sector frame and RMB/CNY locked data.

## Remaining Template Leakage

Remaining repeated text is no longer chart-specific:

- report status explanation
- Money Flow source-policy sentence
- locked data coverage table guidance
- AI self-review table format
- standard safety disclaimer

These are lower severity than the original chart boilerplate because they are either status scaffolding or safety/disclosure text. They still keep broader report-language polish from being perfect.

## Severity Comparison

| Area | Audit 01 | Audit 02 | Audit 03 |
|---|---|---|---|
| Frame-only identity | HIGH | LOW-MEDIUM | LOW-MEDIUM |
| Generic money-flow prose | HIGH | LOW-MEDIUM | LOW-MEDIUM |
| Missing fact-map artifacts | HIGH | PASS | PASS |
| Chart explanation boilerplate | MEDIUM-HIGH | MEDIUM-HIGH | PASS |
| Non-chart scaffolding repetition | MEDIUM | MEDIUM | MEDIUM |

## Final Assessment

Chart explanation leakage is resolved for the 12 representative local reruns. Broader report scaffolding still repeats, but the chart-specific blocker from Audit 02 is closed.

Final status: PASS for chart leakage, WARNING for broader non-chart report scaffolding.

# v5 Template Leakage Audit 02

Date: 2026-05-25

Baseline: `reports/release_checks/v5_0/template_leakage_audit_01.md`

Final status: WARNING
Audit 02 compares the company-specific analysis layer against Audit 01. It verifies that the most severe frame-label and generic money-flow leakage was reduced, while documenting remaining repeated report scaffolding and chart explanation templates.

## Baseline Findings From Audit 01

High-severity findings:

- `company_understanding.rs::understand_company` substituted frame labels into company identity and business-model prose.
- `financial_interpretation.rs::interpret_financials` selected frame-based money-flow paragraphs and injected numbers.
- Reports repeated high-signal analytical phrases across companies.

Repeated Audit 01 phrase:

- `The report should explain how the company earns money before interpreting valuation.`

Medium/high findings:

- Chart explanations repeated across companies.
- Report sections could look like ticker/number swaps.
- Self-review and report scaffolding repeated generic language.

## Audit 02 Scope

Representative reruns:

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

Run ID pattern:

- `company_specific_analysis_01_{ticker_safe}`

Mode:

- local AI fallback
- provider auto
- no external OpenAI API

## Improvements Since Audit 01

### High-Severity Business-Model Phrase Removed

Audit 01 repeated phrase:

- `The report should explain how the company earns money before interpreting valuation.`

Audit 02 result:

- 0/12 rerun reports contain that phrase.

### Company Identity No Longer Frame-Only

Local fallback company identity now includes:

- company name
- provider sector
- provider industry
- provider description snippet
- selected research frame

This is still deterministic fallback prose, but it is no longer only a frame label substitution.

### New Fact-Map Artifacts

Audit 02 found 12/12 reruns generated:

- `metadata/company_fact_sheet.json`
- `metadata/revenue_engine_map.json`
- `metadata/cost_structure_map.json`
- `metadata/capital_allocation_map.json`
- `metadata/money_flow_mechanism.json`
- `metadata/company_specific_questions.json`
- `metadata/template_leakage_check.json`
- `audit/template_leakage_report.md`

### Deterministic Per-Run Template Check

Per-run `metadata/template_leakage_check.json` status:

- PASS: 12/12
- WARNING: 0/12
- FAIL: 0/12

Generic phrase count:

- 0/12 reruns had deterministic generic phrase hits.

## Remaining Template Leakage

Cross-report repetition remains. A repeated sentence scan still found repeated boilerplate in these areas:

- report status explanation
- Money Flow artifact policy text
- chart explanation blocks
- AI self-review table scaffolding

Representative repeated chart phrases still appear across all 12 reruns:

- `What it means: This can show whether the stock has created relative price outperformance or lagged the opportunity-cost benchmark.`
- `What not to overread: A price chart cannot prove the stock is cheap, cannot validate the business model, and cannot replace company-specific cash-flow work.`
- `What it means: The useful question is whether growth is converting into operating profit and cash generation.`
- `What not to overread: A financial trend chart cannot prove segment quality, customer concentration, or management guidance accuracy when those data are missing.`

These phrases are less severe than frame-label identity leakage, but they keep product-quality status at WARNING.

## Severity Comparison

| Area | Audit 01 Severity | Audit 02 Severity | Result |
|---|---|---|---|
| Frame-label company identity | HIGH | LOW-MEDIUM | Improved |
| Generic business-model sentence | HIGH | PASS in reruns | Improved |
| Money Flow sector paragraph risk | HIGH | MEDIUM | Improved with fact-map anchors |
| Missing intermediate fact artifacts | HIGH | PASS | Improved |
| Chart explanation boilerplate | MEDIUM-HIGH | MEDIUM-HIGH | Not fixed in this pass |
| Status/safety boilerplate repetition | LOW | LOW | Acceptable |

## Affected Reports After Audit 02

All 12 rerun reports are affected by remaining chart/status boilerplate repetition, but none triggered the deterministic generic phrase check.

Most important remaining affected area:

- `research-rs/crates/research-report/src/markdown.rs::chart_block`

Lower-severity repeated areas:

- report status explanatory sentence
- AI self-review table format
- legal and provider disclosure text

## Proposed Next Replacement Step

Next pass should target chart and table explanations, not add sector templates.

Recommended architecture:

- `chart_observation_map.json`
  - latest value
  - period change
  - missing chart data
  - source rows
  - unit
  - limitation

- `table_observation_map.json`
  - table rows used
  - source file
  - interpretation boundary
  - company-specific next check

Then report chart prose should be rendered from observations instead of static figure-type paragraphs.

## Final Assessment

Audit 02 confirms that the most severe template leakage from Audit 01 was reduced:

- frame-only business-model phrase removed
- company-specific artifacts generated
- report Money Flow now references locked-data anchors and fact-map artifacts
- deterministic per-run template leakage check passes 12/12

Remaining blocker:

- chart explanations and some scaffolding remain template-like across companies

Final status: WARNING

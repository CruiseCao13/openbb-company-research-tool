# v5 Company-Specific Analysis Engine Pass 01

Date: 2026-05-25

Final status: WARNING
This pass adds a company-specific analysis layer for local fallback runs. It does not call external OpenAI API, does not add sector-template paragraphs, and does not change providers, dashboard styling, PDF output, Tauri, D3, broad_500, or training runs.

The core improvement is that local fallback reports now emit and reference structured company-specific artifacts before report prose:

- `metadata/company_fact_sheet.json`
- `metadata/revenue_engine_map.json`
- `metadata/cost_structure_map.json`
- `metadata/capital_allocation_map.json`
- `metadata/money_flow_mechanism.json`
- `metadata/company_specific_questions.json`
- `metadata/template_leakage_check.json`
- `audit/template_leakage_report.md`

## Files Changed

- `research-rs/crates/research-ai/src/company_understanding.rs`
- `research-rs/crates/research-report/src/renderer.rs`
- `research-rs/crates/research-report/src/markdown.rs`
- `research-rs/crates/research-report/src/tests.rs`
- `reports/release_checks/v5_0/template_leakage_audit_01.md`
- `reports/release_checks/v5_0/template_leakage_audit_02.md`
- `reports/release_checks/v5_0/company_specific_analysis_pass_01.md`

## What Changed

### Company Understanding

The old local fallback identity and business-model phrasing substituted frame labels into generic sentences.

Removed high-severity repeated phrase:

- `The report should explain how the company earns money before interpreting valuation.`

New local fallback identity uses:

- provider company name
- sector / industry
- provider business description snippet
- research frame
- revenue-engine terms
- explicit missing provider fields

### Company-Specific Metadata Artifacts

The renderer now writes fact-map artifacts from locked data and existing AI/local fallback outputs.

Generation flow implemented:

`provider_payload + normalized financials + unit policy + data inventory + evidence map + company_understanding -> company_fact_sheet -> revenue_engine_map -> cost_structure_map -> capital_allocation_map -> money_flow_mechanism -> report sections`

### Report Money Flow

The English report Money Flow section now names the new source artifacts and includes locked-data anchors:

- revenue / operating income base
- operating cash flow
- capex / reinvestment
- free cash flow
- debt / balance-sheet pressure
- working-capital check
- company-specific question

The generic sentence about growth not automatically being valuable was removed from the Money Flow section and replaced with a source-bound policy:

- money-flow interpretation must stay inside fact-map artifacts and explicit data gaps
- unsupported revenue, cost, shareholder-return, or financing claims stay as manual checks

### Template Leakage Check

Each run now writes:

- `metadata/template_leakage_check.json`
- `audit/template_leakage_report.md`

The deterministic check flags:

- generic money-flow/business-model phrases
- repeated non-disclosure analytical sentences inside the report
- lack of company-specific description term reuse

Allowed repeated text:

- legal disclaimer
- provider limitations
- not investment advice
- standard headings
- field labels

## Tests Added / Updated

Added or strengthened tests covering:

- `company_fact_sheet_generated`
- `revenue_engine_map_generated`
- `cost_structure_map_generated`
- `capital_allocation_map_generated`
- `money_flow_mechanism_generated`
- `company_specific_questions_generated`
- `local_fallback_uses_company_fact_sheet`
- `local_fallback_uses_money_flow_mechanism`
- `money_flow_not_only_sector_template`
- `repeated_sentence_patterns_are_flagged`
- `aapl_and_googl_money_flow_not_near_identical`
- `cat_and_isrg_money_flow_not_near_identical`
- `data_limited_company_does_not_get_polished_generic_analysis`
- `template_leakage_report_detects_generic_phrases`

## Samples Rerun

All samples were rerun with:

- `--ai local`
- `--provider auto`
- `--force`
- no external OpenAI API

| Ticker | Run ID | Frame | Report Status | Template Check | New Artifacts |
|---|---|---|---|---|---|
| AAPL | `company_specific_analysis_01_aapl` | Mature Consumer Technology Compounder | PASS | PASS | PASS |
| GOOGL | `company_specific_analysis_01_googl` | Platform Internet / Digital Ads / Cloud | PASS | PASS | PASS |
| CAT | `company_specific_analysis_01_cat` | Cyclical / Industrial Machinery | PASS | PASS | PASS |
| ISRG | `company_specific_analysis_01_isrg` | Medical Devices / Surgical Robotics | PASS | PASS | PASS |
| JPM | `company_specific_analysis_01_jpm` | Financials / Bank-like Screening | PASS | PASS | PASS |
| RKLB | `company_specific_analysis_01_rklb` | Speculative Aerospace / Space Systems | WARNING | PASS | PASS |
| 600519.SH | `company_specific_analysis_01_600519_sh` | A-share Premium Baijiu / Consumer Brand | PASS | PASS | PASS |
| 000001.SZ | `company_specific_analysis_01_000001_sz` | Financials / Bank-like Screening | PASS | PASS | PASS |
| 300750.SZ | `company_specific_analysis_01_300750_sz` | New Energy / Battery Manufacturing | PASS | PASS | PASS |
| 601318.SH | `company_specific_analysis_01_601318_sh` | Insurance / Integrated Financials | PASS | PASS | PASS |
| 600276.SH | `company_specific_analysis_01_600276_sh` | Pharma / Innovative Drug Portfolio | PASS | PASS | PASS |
| 601899.SH | `company_specific_analysis_01_601899_sh` | Mining / Nonferrous Metals / Commodity Cycle | PASS | PASS | PASS |

RKLB remains report-level WARNING because local fallback is intentionally data-limited for speculative aerospace. It did not fail template leakage.

## Generated Artifact Status

Required new metadata artifacts were present for 12/12 representative reruns.

For each sample:

- `metadata/company_fact_sheet.json`: generated
- `metadata/revenue_engine_map.json`: generated
- `metadata/cost_structure_map.json`: generated
- `metadata/capital_allocation_map.json`: generated
- `metadata/money_flow_mechanism.json`: generated
- `metadata/company_specific_questions.json`: generated
- `metadata/template_leakage_check.json`: generated
- `audit/template_leakage_report.md`: generated

## Before / After Template Leakage Summary

Before:

- high-severity company identity was largely frame-label substitution
- business-model text repeated `The report should explain how the company earns money before interpreting valuation.`
- Money Flow relied on frame paragraphs and generic connective prose
- chart explanation blocks were repeated across companies

After:

- old high-severity business-model phrase appears in 0/12 rerun reports
- generic phrase count in `template_leakage_check.json` is 0/12
- deterministic template leakage status is PASS for 12/12 reruns
- reports now include locked-data anchors and fact-map source artifacts
- cross-report chart/status boilerplate still repeats

## Repeated Phrase Count

Representative cross-report scan after rerun:

- Old high-severity phrase: 0 occurrences in the 12 rerun reports
- Deterministic generic phrase count: 0/12 reports
- Cross-report repeated analytical/boilerplate sentences >= 3 occurrences: 32

Remaining repeated text is concentrated in:

- report status explanatory boilerplate
- Money Flow artifact policy sentence
- chart explanation blocks
- AI self-review table scaffolding

This is why final status remains WARNING.

## Company-Specific Facts Used

The new artifacts use these locked-data anchors where present:

- provider profile name, sector, industry, description
- market and currency
- provider source and adapter
- revenue / total revenue / 营业收入
- gross profit / 毛利
- operating income / 营业利润
- net income / 归母净利润 / 净利润
- operating cash flow / 经营现金流
- capex / capital expenditure / 资本开支
- free cash flow / 自由现金流
- debt / borrowings / 有息负债 / 负债合计
- inventory / 存货
- receivables / 应收
- cash / 货币资金
- bank metrics such as ROE, ROA, NIM, NPL, capital adequacy where provider supplies them

## Remaining Generic Phrases / Warnings

Remaining template risk:

- Chart explanations still explain chart type more than actual company chart movement.
- Report status and safety boilerplate repeat by design.
- AI self-review table scaffolding repeats.
- The report still includes a repeated Money Flow artifact-policy sentence across all companies.

No hard template leakage failure remained in the deterministic per-run scan.

## Validation Commands

Focused validation already passed:

- `cargo test --manifest-path research-rs/Cargo.toml -p research-ai -p research-report`

Full validation was run after this report generation and is recorded in the final response.

## Final Assessment

The pass materially improves local fallback specificity and adds the requested intermediate artifacts. It should not be treated as the final architecture, because chart explanations and some report scaffolding are still too repetitive.

Final status: WARNING

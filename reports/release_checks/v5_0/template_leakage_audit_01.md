# v5 Template Leakage Audit 01

Date: 2026-05-25

Scope: local fallback and report generation code only. This audit did not add sector templates, did not run external AI, did not modify providers, and did not change report generation behavior.

Audit status: PASS
Product-quality status: WARNING

The audit found meaningful template leakage in the v5 local fallback path. Recent money-flow work improved data grounding, but several functions still select frame-based paragraphs and then substitute ticker, frame label, or locked-data numbers. This creates reports that can look company-specific while still being template-driven.

## Files and Reports Reviewed

Source areas reviewed:

- `research-rs/crates/research-ai/src/company_understanding.rs`
- `research-rs/crates/research-ai/src/financial_interpretation.rs`
- `research-rs/crates/research-report/src/markdown.rs`
- `research-rs/crates/research-report/src/renderer.rs`
- `research-rs/crates/research-report/src/dashboard.rs`
- `providers/chart_provider.py`

Representative generated reports reviewed:

- `reports/AAPL/runs/money_flow_specificity_01_aapl/report/AAPL_research_report.md`
- `reports/GOOGL/runs/money_flow_specificity_01_googl/report/GOOGL_research_report.md`
- `reports/CAT/runs/money_flow_specificity_01_cat/report/CAT_research_report.md`
- `reports/ISRG/runs/money_flow_specificity_01_isrg/report/ISRG_research_report.md`
- `reports/JPM/runs/money_flow_specificity_01_jpm/report/JPM_research_report.md`
- `reports/600519.SH/runs/core_data_accuracy_rerun_01_600519_sh/report/600519.SH_research_report.md`
- `reports/000001.SZ/runs/core_data_accuracy_rerun_01_000001_sz/report/000001.SZ_research_report.md`
- `reports/300750.SZ/runs/core_data_accuracy_rerun_01_300750_sz/report/300750.SZ_research_report.md`
- `reports/601318.SH/runs/core_data_accuracy_rerun_01_601318_sh/report/601318.SH_research_report.md`
- `reports/600276.SH/runs/core_data_accuracy_rerun_01_600276_sh/report/600276.SH_research_report.md`
- `reports/601899.SH/runs/core_data_accuracy_rerun_01_601899_sh/report/601899.SH_research_report.md`

## Template-Heavy Functions

### `company_understanding.rs::understand_company`

Severity: HIGH

This function contains deterministic frame selection and fallback company identity text that can substitute a sector frame for real business understanding.

Template leakage examples:

- `{name} is best treated as {frame} based on the locked provider profile and financial context.`
- `The research frame is {frame}. The report should explain how the company earns money before interpreting valuation.`

Why it matters:

- A frame label can become the company identity.
- A report can pass with a plausible label but thin product, customer, geography, or revenue-engine detail.
- The local fallback path still depends heavily on hardcoded frame guards and sector branches.

Required replacement direction:

- Build `company_fact_sheet` first.
- Derive frame from facts, conflicts, and evidence.
- Reject identity text that only restates the frame.

### `financial_interpretation.rs::interpret_financials`

Severity: HIGH

This is the main source of local fallback money-flow prose. It now uses locked data more often, but the architecture remains paragraph-template driven by industry frame.

Template leakage examples:

- `Money comes from bank economics...`
- `Money comes from platform revenue streams...`
- `Money comes from the hardware-plus-services ecosystem...`
- `Money is absorbed by...`
- `Money comes from the revenue engines implied by the {frame} frame...`
- `Debt-like obligations appear in locked data around...; financing risk should be reviewed in filings.`
- `Valuation should fit {frame}. The report must not force PE, PS, or FCF yield...`

Why it matters:

- The output can become a sector paragraph with numbers injected.
- Company-specific operating mechanics are inferred from frame labels rather than explicit facts.
- Local fallback can still sound generic even when money-flow audits pass.

Required replacement direction:

- Generate `revenue_engine_map`, `cost_structure_map`, `capital_allocation_map`, and `money_flow_mechanism`.
- Render money-flow prose from those maps.
- Require every revenue/cost/cash-flow sentence to cite a map field or an explicit data gap.

### `markdown.rs::render_report`

Severity: MEDIUM-HIGH

Report sections and chart explanations repeat strongly across companies. Some repetition is expected for report structure, but chart interpretation text is too generic and often does not describe company-specific chart movement.

Repeated chart explanation examples:

- `What to look at: Compare the company price path with the benchmark...`
- `What it means: This can show whether the stock has created relative price outperformance...`
- `What not to overread: A price chart cannot prove the stock is cheap...`

Why it matters:

- Chart cards can look explanatory while describing chart type rather than the specific chart.
- Users may not learn what the actual chart shows for the selected company.

Required replacement direction:

- Chart explanations should be generated from chart metadata plus computed chart observations.
- Each chart needs actual latest value, period movement, missing-data state, and limitation.

### `renderer.rs`

Severity: MEDIUM

Several renderer helpers create repeated audit/report scaffolding. This is less dangerous than frame leakage, but it contributes to reports feeling mechanically assembled.

Affected areas:

- `write_data_inventory`
- `write_chart_plan`
- `write_table_plan`
- `write_money_flow_map`
- `write_core_data_accuracy_audits`

Repeated patterns:

- `No critical data gaps were flagged by the current blueprint.`
- repeated chart/table planning language
- repeated locked-data appendix phrasing

Required replacement direction:

- Keep structural labels, but make body rows and explanation text derive from structured fact maps.
- Flag sections that contain only generic scaffolding and no company-specific fact reference.

### `dashboard.rs::render_company_dashboard`

Severity: LOW-MEDIUM

Dashboard status cards repeat provider, mock, and local fallback disclosure text. Some of this is appropriate because it is system UI. The risk is lower than in report prose, but it should remain clearly diagnostic and not masquerade as company analysis.

Required replacement direction:

- Keep repeated safety badges.
- Avoid using dashboard card text as analytical prose.

## Repeated Phrases Found

High-signal repeated phrases across the representative report set:

| Count | Phrase | Severity | Notes |
|---:|---|---|---|
| 11 | `The report should explain how the company earns money before interpreting valuation.` | HIGH | Frame-label substitution, not business understanding. |
| 11 | `This matters because growth is not automatically valuable.` | MEDIUM | True but generic across companies. |
| 11 | `The report needs to distinguish operating cash generation from financing, reinvestment, R&D, capex, working capital, buybacks, and debt service.` | MEDIUM-HIGH | Good checklist, weak company specificity. |
| 11 | `What to look at: Compare the company price path with the benchmark...` | MEDIUM-HIGH | Chart-type explanation repeated across companies. |
| 11 | `What not to overread: A price chart cannot prove the stock is cheap...` | LOW-MEDIUM | Useful caution, but too generic if every chart uses it. |
| 11 | `The report does not provide a target price, buy/sell recommendation, or short-term price prediction.` | LOW | Acceptable repeated disclaimer. |
| 6 | A-share public endpoint limitation text | LOW | Acceptable if used as provider disclosure. |

Structural headings such as `Report Status`, `Company Identity`, `Money Flow`, and `AI Self Review` also repeat, but these are intentional report format and are not considered leakage by themselves.

## Affected Reports

Severity: MEDIUM-HIGH across the reviewed local fallback reports.

Affected reports share common structure, disclaimers, chart explanations, and frame-derived business-model language. The most affected analytical sections are:

- Company Identity
- Business Model
- Money Flow
- Financial Statement Interpretation
- Chart explanation blocks
- AI Self Review template fields

Reports most exposed to template leakage:

- AAPL local fallback report: mature consumer technology frame is reasonable, but identity text still relies on frame substitution.
- GOOGL local fallback report: platform internet language is plausible but repeated chart and money-flow scaffolding remain generic.
- CAT local fallback report: industrial cycle frame works, but working-capital/capex mechanics need fact-map rendering rather than paragraph selection.
- ISRG local fallback report: medical device frame works, but product/procedure-linked revenue logic should come from a revenue-engine map.
- JPM local fallback report: bank frame works, but local fallback should avoid generic checklist language and use bank-specific facts only when present.
- CN A-share reports: provider/source disclosure is clean, but sector frame text often reads like deterministic guard output rather than company-specific business understanding.

## Severity Summary

HIGH:

- Frame label substituted for company identity or business model.
- Money-flow paragraphs selected by frame and then filled with numbers.
- Local fallback functions that can produce plausible prose without proving company-specific facts.

MEDIUM-HIGH:

- Chart explanations explain chart type instead of chart contents.
- Reports can resemble ticker/number swaps when local fallback is used.
- Self-review fields repeat generic checks and may not challenge template-derived assertions.

MEDIUM:

- Generic report guidance phrases repeated across companies.
- Data inventory and chart-plan scaffolding can dominate content.

LOW:

- Legal disclaimers.
- Provider limitation badges.
- Report section headings.

## Proposed Replacement Architecture

Do not add more sector templates as the next fix. The replacement should move from paragraph selection to structured fact maps.

### `company_fact_sheet`

Purpose: a compact, source-backed identity object.

Fields:

- ticker
- company_name
- market
- provider
- source
- sector
- industry
- business_description
- products_or_services
- customers_or_end_markets
- geography
- operating_model_keywords
- forbidden_frames
- confidence
- data_gaps
- evidence_refs

Rule:

- `company_identity` cannot be rendered from frame alone.
- If products/services or customer/end-market evidence is absent, the report must say the identity is data-limited.

### `revenue_engine_map`

Purpose: describe how money enters the company, with source-backed confidence.

Fields:

- revenue_engine
- supporting_fields
- latest_revenue_value
- segment_or_product_evidence
- customer_or_contract_evidence
- confidence
- unsupported_terms
- missing_fields
- evidence_refs

Rule:

- No sentence may claim a revenue engine unless it appears in this map or is explicitly marked as a hypothesis/data gap.

### `cost_structure_map`

Purpose: describe what absorbs operating cash.

Fields:

- cost_driver
- data_field
- latest_value
- trend
- applicability_by_company_type
- missing_fields
- evidence_refs

Examples:

- COGS / gross margin
- R&D
- SG&A
- capex
- working capital
- credit loss
- insurance claims
- inventory
- receivables

Rule:

- Banks and insurers should not be forced into industrial FCF language unless the report explains why that metric is secondary.

### `capital_allocation_map`

Purpose: describe where cash goes after operations.

Fields:

- capex
- dividends
- buybacks
- debt issuance
- debt repayment
- equity issuance
- acquisitions
- cash balance
- financing_gap
- evidence_refs

Rule:

- Buyback/dividend/debt claims are forbidden unless supported by locked data or marked as missing.

### `money_flow_mechanism`

Purpose: render a source-backed flow of cash through the company.

Suggested structure:

1. Source: revenue/capital inflow.
2. Conversion: margin, OCF, credit loss, claims, or working capital.
3. Absorption: capex, R&D, inventory, receivables, debt, distributions.
4. Gap: what is missing or uncertain.
5. Next check: exact filing/provider field to verify.

Rule:

- Money Flow prose should be generated from this mechanism, not from sector paragraphs.
- If the mechanism is incomplete, report status should not present Money Flow as fully specific.

### `company_specific_questions`

Purpose: force the local fallback to ask questions that only make sense for this company.

Examples:

- For CAT: is dealer/inventory pressure rising relative to revenue?
- For JPM: are credit costs and deposits moving against NIM?
- For RKLB/LUNR: is cash burn covered by contracts, backlog, or financing?
- For 300750.SZ: are inventory and receivables absorbing cash as EV battery demand cycles?
- For 601318.SH: is profit driven by underwriting, investment income, or capital-market marks?

Rule:

- Questions must reference company facts, missing fields, or industry-specific mechanics.
- Generic questions such as `monitor capital allocation` should fail specificity checks.

## Recommended Next Pass

Recommended next pass: `v5 Fact Map Architecture Pass 01`.

Objectives:

1. Add `company_fact_sheet.json`.
2. Add `revenue_engine_map.json`.
3. Add `cost_structure_map.json`.
4. Add `capital_allocation_map.json`.
5. Add `money_flow_mechanism.json`.
6. Add `company_specific_questions.json`.
7. Make local fallback render Money Flow and Business Model from these artifacts.
8. Add leakage tests:
   - `company_identity_not_frame_label_only`
   - `money_flow_sentences_have_fact_map_refs`
   - `chart_explanation_uses_chart_observations`
   - `company_specific_questions_not_generic`
   - `local_fallback_report_similarity_below_threshold`

Acceptance:

- A report should not pass if its company identity, business model, or money flow can be produced by swapping only ticker, frame, and numbers.

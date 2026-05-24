# Known Issues and Future Roadmap

## v4.3 Status Note

v4.3 adds asset-aware routing, thesis spine generation, profile-specific interpretation blocks, AI interpretation patch artifacts, organized report packs, and automatic self-review files.

This reduces the risk that a speculative-growth company receives mature-compounder language, but it does not mean the system fully covers every industry. Low-confidence or specialized companies should still be downgraded to screening-only when sector-specific data is missing.

Remaining future work:

- deeper industry frameworks for biotech, REITs, insurance, shipping, utilities, and other specialized industries
- deeper semiconductor sub-frameworks for IDM, foundry, fabless, memory, equipment, and cyclical semi-capex names
- stronger numeric-claim alignment lint after AI patching
- more peer-relative and industry-normalized scoring
- broader random-company pressure testing
- optional Rust report-pack checker only after measuring a real local bottleneck

Recent INTC-like stress testing showed that a generic `Technology / Screening Only` downgrade is not enough when the system has enough clues to form a better industry hypothesis. The router now includes a generalized `Capital-Intensive Semiconductor Turnaround` profile and framework-gap reports must name foundry, capex, gross-margin, process-node, data-center, and free-cash-flow verification needs when those clues appear.

This document records the current limitations, known risks, and future development directions of `openbb-company-research-tool`.

The purpose is not to solve every issue immediately. The purpose is to keep the project honest, maintainable, and methodologically clear.

---

## 1. Data Reliability Issues

### 1.1 Public Data Source Limitations

The tool relies on public data providers such as `yfinance` and OpenBB.

Known risks:

- delayed data
- missing fields
- stale values
- inconsistent financial statement coverage
- unreliable segment data
- weaker data quality for small-cap, distressed, foreign, or newly listed companies

### User Risk

A report may look precise even when the underlying public data is incomplete.

### Future Direction

Create a Data Reliability Layer that can identify stale, missing, inconsistent, or low-confidence financial data before the report presents derived conclusions.

---

### 1.2 Cross-Statement Time Mismatch

Financial ratios may combine data from different statements:

- income statement
- balance sheet
- cash flow statement

If these statements come from different reporting periods, ratios such as Net Debt / EBITDA or Debt / FCF may become misleading.

### Example Risk

Balance sheet data may be updated to the latest quarter while cash flow data still reflects an older period.

### Future Direction

Add a Time-Sync Audit before calculating cross-statement metrics.

Potential rule:

- if statement dates differ by more than one quarter, block or downgrade affected calculations

---

### 1.3 Lookback and Survivorship Bias

The tool usually works with currently retrievable tickers.

This creates possible lookback and survivorship bias:

- delisted companies may be missing
- catastrophic historical periods may be truncated
- distressed assets may have incomplete data
- historical risk may look safer than it actually was

### User Risk

A user may believe historical drawdown or volatility was moderate, when the most dangerous period was not captured by the available dataset.

### Future Direction

Add warnings for:

- short price history
- missing historical periods
- delisted or renamed tickers
- unusually sparse price data
- incomplete reporting history

---

## 2. Report Language and Localization Issues

### 2.1 English Report Quality

The English report is usable but can still sound generic in some sections, especially AI Review.

Common risks:

- safe summary language
- repeated checklist language
- weak synthesis of the central tension
- generic caution instead of specific reasoning

### Future Direction

Improve AI Review prompting to focus on:

- central tension
- strongest evidence
- weakest assumption
- most likely beginner misreading
- top 3-5 verification items

---

### 2.2 Chinese Report Mode Not Yet Implemented

The project currently does not support a true Chinese report mode.

A future Chinese mode must not be a direct translation of the English report.

### Problem

A literal Chinese translation would likely create:

- translationese
- awkward financial wording
- excessive jargon
- unnatural sentence structure
- poor beginner readability
- loss of the original reasoning flow

### Required CLI Direction

Future Chinese mode should support:

```bash
--language zh
--cn
--chinese
```

Recommended design:

- `--language zh` should be the formal option.
- `--cn` and `--chinese` should be aliases.
- Do not use misspelled options such as `--chineses`.
- File names may remain English for compatibility.
- Report headings and explanations should become Chinese in zh mode.
- Metric names may keep common English abbreviations where useful, such as FCF, EBITDA, PE, and EV/EBITDA, but each should be explained naturally.

### Target Standard

The Chinese report should be written in natural Chinese logic, not translated English.

It should be:

- clear
- direct
- beginner-friendly
- analytically disciplined
- non-sensational
- non-promotional
- free of machine-translation tone

### Desired Chinese Style

The Chinese report should explain financial logic in plain Chinese without losing rigor.

Preferred style:

```text
这份报告真正想说的不是 AAPL 差，而是：它是一家现金流很强的成熟公司，但当前价格已经包含了不少乐观预期。问题不在于公司能不能赚钱，而在于未来的增长和利润率能不能支撑现在的估值。
```

Avoid over-emotional or unsupported language such as:

- 暴跌
- 割韭菜
- 贵得离谱
- 伸手要钱
- 画饼
- 绝对安全
- 必然上涨
- 马上爆发

Unless directly supported by data and still phrased carefully.

### Future Direction

Implement v3.1 as:

```text
v3.1 — Chinese Report Mode and Localized AI Review
```

Potential scope:

- add `--language {en,zh}`
- add `--cn`
- add `--chinese`
- generate native Chinese reports
- support Chinese AI Review output
- create `examples/sample_reports/AAPL_sample_research_report_zh.md`
- preserve English mode unchanged

---

### 2.3 Avoiding Financial Media Slang

Plain language is important, but exaggerated phrasing should be avoided.

Avoid:

- sensational risk language
- emotional market commentary
- clickbait expressions
- unsupported claims of collapse, bubble, or disaster

Preferred style:

- clear
- restrained
- specific
- evidence-based

---

## 3. AI Review Limitations

### 3.1 Summary vs. Review

Current AI Review can sometimes behave like a summarizer instead of a critical reviewer.

### Risk

The AI may repeat the report rather than audit the logic.

### Future Direction

Strengthen prompt design so AI Review focuses on:

- whether the evidence supports the conclusion
- where the report may overstate confidence
- what contradiction exists in the data
- what a beginner might misunderstand
- what must be verified next

---

### 3.2 Verification List Redundancy

The AI Review's `What to Verify Next` section can duplicate Manual Verification.

### Risk

The report becomes longer without becoming clearer.

### Future Direction

Limit AI Review verification items to the top 3-5 most important uncertainties for that specific company.

---

### 3.3 Action Boundary

The AI layer must explain risk without giving investment instructions.

Allowed:

- explain why drawdown matters
- explain why valuation is demanding
- explain why cash flow quality matters
- explain what needs verification

Not allowed:

- buy / sell / hold recommendations
- price targets
- short-term price predictions
- portfolio allocation instructions
- direct trade management instructions

---

## 4. Scoring Methodology Issues

### 4.1 Research Score Is Heuristic

The Research Score is a screening heuristic.

It is not:

- a valuation model
- a prediction model
- an alpha model
- a statistically validated investment signal

### Risk

A precise score such as `61.34 / 100` may create false certainty.

### Future Direction

Explore ways to reduce false precision, such as:

- confidence bands
- qualitative score labels
- data-confidence-adjusted scoring
- peer-relative scoring

---

### 4.2 Cross-Category Comparability

A score for a mature company and a score for a speculative growth company should not be interpreted in the same way.

### Example

A 61 score for Apple and a 61 score for Rocket Lab may represent very different risk profiles.

### Future Direction

Make score interpretation more explicit by company type:

- Mature Compounder
- Profitable Growth
- Speculative Growth
- Financials
- Cyclical
- ETF
- Data Limited

---

### 4.3 Better Methodology Still Needs Research

Potential future directions:

- peer-relative scoring
- factor-inspired scoring
- quality / growth / value / momentum / risk structure
- industry-normalized Z-scores
- scenario-based scoring
- trend-based scoring
- data-confidence-weighted scoring
- red-flag rule system

These should be researched before implementation.

---

## 5. Chart and Visualization Issues

### 5.1 Chart Purpose

The project already generates multiple charts, but future charts should not be added just for quantity.

Each chart must answer a specific question.

Examples:

- Price vs Benchmark: did the stock outperform?
- Drawdown: how painful was the ride?
- Revenue vs FCF: did growth become cash?
- Margin Trend: is profitability improving?
- Debt vs Cash Flow: is the balance sheet fragile?
- Valuation Multiples: is the market already pricing in too much?

---

### 5.2 Unit Standardization

Charts should use consistent units.

Current and future risks:

- raw numbers too large to read
- inconsistent billion / million labels
- scientific notation
- unclear y-axis units
- unclear percentage scales

### Future Direction

Add a chart unit registry that standardizes:

- USD billions
- USD millions
- percentages
- ratios
- indexed performance values

---

### 5.3 Visual Design System

Charts need a more explicit visual standard.

Future requirements:

- consistent color palette
- consistent grid style
- consistent font sizes
- high DPI export
- clear titles
- subtitle with period / benchmark / data source
- chart-reading notes
- no unnecessary clutter

---

### 5.4 HTML Dashboard Portability

Current HTML dashboard works well as a local output.

However, if users copy the Markdown report or move files into another platform, links may break.

### Future Direction

Explore portable publishing modes:

- self-contained HTML report
- base64 embedded images
- exportable ZIP bundle
- GitHub Pages publishing mode
- Ghost / blog-ready output mode

---

## 6. Engineering Reliability Issues

### 6.1 API and Data Provider Rate Limits

High-frequency runs or multi-ticker usage may trigger rate limits from:

- yfinance
- OpenBB providers
- OpenAI API

### Future Direction

Potential improvements:

- retry with exponential backoff
- jittered delay
- per-provider request counters
- clearer terminal warning
- skip AI Review for batch runs unless explicitly requested
- cache repeated data calls

---

### 6.2 Dependency Drift

The project depends on libraries that can change over time:

- openbb
- yfinance
- openai
- pydantic
- rich
- plotly
- pandas
- matplotlib

### Risk

A future library update may break installation, data fetching, API calls, or report rendering.

### Future Direction

Consider:

- tested dependency versions
- requirements lock file
- uv / poetry / pip-tools workflow
- release-specific dependency snapshot
- CI installation test

---

### 6.3 Project Complexity

The project now includes:

- deterministic financial calculations
- reports
- charts
- interactive HTML
- AI Review
- Rich terminal UI
- tests
- docs
- samples

### Risk

Future changes may blur module boundaries.

### Future Direction

Maintain clear module ownership:

- `company_research_tool.py`: deterministic orchestration and report data assembly
- `ai_review.py`: AI schema, prompt, API call, fallback, and AI Review rendering
- `terminal_ui.py`: CLI display and terminal interaction
- `docs/`: methodology, limitations, development notes, and roadmap
- `examples/`: sample outputs only

Avoid mixing AI logic, chart logic, data fetching, and terminal rendering inside one file when future changes are made.

---

## 7. Report Presentation Issues

### 7.1 Minor Formatting Issues

Known presentation issues found after v3.0:

- duplicate horizontal separators before some sections
- provider business summaries may truncate mid-word
- some table labels still expose raw provider field names
- AI Review may repeat manual verification items
- some sections may feel longer than necessary for beginners

### Future Direction

Improve the report rendering layer without changing core calculations:

- clean section separators
- truncate long provider text at sentence or word boundaries
- map raw provider fields to human-readable labels
- keep AI Review concise
- reduce repeated verification lists
- preserve professional tone

---

### 7.2 Raw Provider Field Names

Some valuation or financial table fields may appear as raw provider keys, such as:

- `trailingPE`
- `forwardPE`
- `priceToSalesTrailing12Months`
- `enterpriseToRevenue`
- `enterpriseToEbitda`
- `operatingCashflow`

### User Risk

Raw provider names make the report feel unfinished and harder for beginners to read.

### Future Direction

Add a display label registry for report tables.

Example mapping:

| Raw Field | Display Label |
|---|---|
| `trailingPE` | Trailing P/E |
| `forwardPE` | Forward P/E |
| `priceToSalesTrailing12Months` | Price / Sales |
| `enterpriseToRevenue` | EV / Revenue |
| `enterpriseToEbitda` | EV / EBITDA |
| `operatingCashflow` | Operating Cash Flow |

Internal keys can remain unchanged. Only the display layer should change.

---

## 8. Product Positioning Boundaries

### 8.1 Not a Stock-Picking Tool

The project must not be positioned as:

- a stock picker
- a trading signal generator
- a short-term prediction tool
- a portfolio allocation system
- a guaranteed return system

### Correct Positioning

The project is a first-pass company research workflow.

It helps users:

- organize public market and financial data
- compare a stock against a benchmark
- identify basic business and valuation risks
- understand possible beginner misreadings
- generate reviewable research records
- avoid purely emotional stock selection

### Core Principle

> Data calculates. AI reviews. Human decides.

---

### 8.2 Beginner-Friendly Does Not Mean Oversimplified

The tool should help beginners understand financial concepts, but it should not become childish, sensational, or misleading.

Avoid:

- clickbait language
- exaggerated downside claims
- unsupported certainty
- emotional market commentary
- mockery of beginner users

Preferred style:

- clear
- direct
- restrained
- specific
- evidence-based
- beginner-aware

---

## 9. Future Version Roadmap

### v3.0.1 — Documentation and Report Polish

Potential scope:

- add `docs/development_log.md`
- add `docs/known_issues_and_roadmap.md`
- fix duplicate report separators
- clean provider summary truncation
- improve display labels
- limit AI Review verification items to top 3-5
- preserve v3.0 architecture

This should be a small polish release, not a new feature expansion.

---

### v3.1 — Chinese Report Mode and Localized AI Review

Potential scope:

- add `--language zh`
- add `--cn`
- add `--chinese`
- generate native Chinese reports
- avoid literal translation from English
- preserve analytical rigor
- use natural Chinese headings and explanations
- support Chinese AI Review output
- add `AAPL_sample_research_report_zh.md`

Core standard:

> Professional analysis, plain Chinese.

The Chinese version should explain financial logic clearly without sounding like machine translation or financial media hype.

---

### v3.2 — Data Reliability Layer

Potential scope:

- cross-statement time-sync audit
- stale data detection
- missing critical field severity
- calculation blocking when data is unsafe
- Data Limited classification escalation
- raw data snapshot preservation
- improved data confidence scoring

This is the most important future reliability upgrade.

---

### v3.3 — Chart Design System

Potential scope:

- chart unit registry
- consistent visual theme
- conclusion-first chart titles
- better y-axis formatting
- improved annotations
- cleaner Plotly dashboard layout
- chart-reading notes that explain both what the chart shows and what it does not prove

---

### v4.0 — Methodology Upgrade

Potential scope:

- peer-relative scoring
- industry-normalized metrics
- factor-inspired model structure
- quality / growth / value / momentum / risk framework
- scenario-based scoring
- confidence-adjusted scores
- red-flag rule system
- formal methodology documentation

This should only be attempted after the current heuristic model is fully documented and its limitations are clear.

---

## 10. Current Priority

Do not expand the system before documenting and prioritizing its limitations.

Immediate priority:

1. Keep v3.0 stable.
2. Record known issues.
3. Add development log.
4. Write project narrative.
5. Only then decide the next engineering step.

---

## 11. v4.0 AI Governance Settlement

v4.0 redefines the AI layer as a bounded second-pass analyst, not a generic summary writer.

Core rule:

> Hard rules prevent hallucination. AI Review prevents mediocrity. Language lint prevents AI flavor. Failure naming prevents misuse.

The project should use a four-gate workflow:

1. Data Audit Gate
2. Risk Method Gate
3. AI Analyst Review Gate
4. Language Lint Gate

The AI Analyst Review Gate exists to find weak reasoning, missing evidence, overconfident conclusions, unanswered questions, and next verification steps. It must not calculate metrics, mutate deterministic values, fabricate external facts, or provide buy/sell recommendations.

### 11.1 AI Analyst Review Gate

The AI Analyst Review Gate must generate:

- `ai_correction_log.md`
- optionally `ai_correction_log.json`

Required sections:

- Report Corrections
- Unanswered Questions
- Answerability Classification
- Research Stance
- Next 3 Checks

Each correction must include:

- `section`
- `original_issue`
- `suggested_revision`
- `reason`
- `requires_data_verification`
- `severity: LOW / MEDIUM / HIGH`
- `evidence_boundary: FROM_PAYLOAD / NEEDS_EXTERNAL_VERIFICATION / NOT_ALLOWED`

The AI layer may propose corrections to interpretation and wording. It must never overwrite deterministic payload values such as revenue, free cash flow, PE, Sharpe, beta, or the Research Score.

### 11.2 Answerability Classification

Every important question should be classified before it is answered.

Allowed statuses:

- `ANSWERABLE_FROM_REPORT`
- `PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION`
- `NOT_ANSWERABLE_FROM_CURRENT_DATA`

Rules:

- Future price, target price, next-year outperformance, and short-term return questions must be `NOT_ANSWERABLE_FROM_CURRENT_DATA`.
- Causal `why` questions without direct payload evidence must be `PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION`.
- Segment-related claims cannot be treated as evidence when segment data is missing.

Required stance language:

```text
This is not a buy/sell recommendation. It is a research stance.
```

Chinese mode must include:

```text
这不是买卖建议，而是研究立场。
```

`Next 3 Checks` must contain exactly three concrete, data-driven, non-generic verification actions.

### 11.3 Missing Metric Placeholder Control

Loose missing-value strings are not allowed in the structured payload or base report draft.

Forbidden:

- `N/A`
- `Data not found`
- `Unknown`
- `None`

Required registered placeholders:

- `[METRIC_MISSING_RAW]`
- `[FIELD_MISSING_PROVIDER]`
- `[PRIMARY_SOURCE_REQUIRED]`
- `[SEGMENT_DATA_MISSING]`
- `[METHOD_ASSUMPTION_MISSING]`
- `[PRICE_LABEL_UNVERIFIED]`

These placeholders must remain unchanged in both English and Chinese pipelines. They must not be translated before being passed into prompts or rule checks.

The rendering layer may map them into natural English or Chinese, but the structured payload must keep the original enum placeholder.

If the AI Analyst Review Gate sees one of these placeholders, it must not infer business facts from it. It must classify the related question as:

- `PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION`
- or `NOT_ANSWERABLE_FROM_CURRENT_DATA`

and set evidence boundary to:

- `NEEDS_EXTERNAL_VERIFICATION`
- or `NOT_ALLOWED`

### 11.4 Self-Correction Loop Limits

The AI layer must not recursively trigger itself.

Required limits:

- `max_correction_passes = 1`
- `max_language_rewrite_attempts = 3`

The AI Analyst Review Gate can produce one bounded correction pass. The Language Lint Gate may rewrite offending language up to three times. If it still fails, the report must expose the failure instead of pretending success.

### 11.5 Failure Naming Chain

If any gate returns `FAIL`, the final report must be marked `UNVERIFIED`.

Report filenames must be prefixed:

- `UNVERIFIED_AAPL_research_report.md`
- `UNVERIFIED_AAPL_research_report_cn.md`

Auxiliary gate logs from the same run must use the same prefix:

- `UNVERIFIED_data_audit.md`
- `UNVERIFIED_data_audit.csv`
- `UNVERIFIED_ai_correction_log.md`
- `UNVERIFIED_ai_correction_log.json`
- `UNVERIFIED_language_lint_report.md`
- `UNVERIFIED_price_label_sanity_check.md`

This makes failed runs visible in file browsers, scripts, archives, and CI checks.

Metadata must include:

- `DATA_AUDIT_STATUS: PASS / WARNING / FAIL`
- `RISK_METHOD_STATUS: PASS / WARNING / FAIL`
- `AI_ANALYST_REVIEW_STATUS: PASS / WARNING / FAIL`
- `LANGUAGE_LINT_STATUS: PASS / WARNING / FAIL`
- `OVERALL_REPORT_STATUS: VERIFIED / WARNING / UNVERIFIED`

Overall status rules:

- all gates `PASS` -> `VERIFIED`
- any `WARNING`, no `FAIL` -> `WARNING`
- any `FAIL` -> `UNVERIFIED`

If the report is `UNVERIFIED`, the CLI must print a clear warning and point users to:

- `data_audit.md`
- `ai_correction_log.md`
- `language_lint_report.md`
- `price_label_sanity_check.md`

### 11.6 Research Battle Card Remains Required

The Research Battle Card remains part of the final report. The AI Correction Log is not a replacement for it.

Difference:

| Module | Purpose |
|---|---|
| Research Battle Card | Final report research storyline |
| AI Correction Log | Backstage challenge layer, answerability audit, and verification prioritization |

Research Battle Card sections:

- The Long Bet / 买入的核心赌注
- The Short Trigger / 做空或离场的死穴
- Market Pricing / 市场已经交易了什么
- What Must Hold / 什么必须守住
- Kill Criteria / 一票否决条件
- Verification Priority / 最优先核查的 3 件事

Limits:

- The Long Bet: max 3 sentences
- The Short Trigger: max 3 sentences
- Market Pricing: max 3 sentences
- What Must Hold: max 5 bullets
- Kill Criteria: max 5 bullets
- Verification Priority: exactly 3 items

---

## Summary

The project is now a usable and presentable company research workflow, but it is not a finished investment product.

Its biggest remaining challenges are:

1. data reliability
2. natural report language
3. Chinese report mode and localization
4. AI Review depth and precision
5. scoring methodology limits
6. chart clarity and portability
7. dependency and runtime reliability
8. module boundary discipline

The purpose of this roadmap is to prevent uncontrolled feature expansion and keep the project honest about its current strengths and limitations.

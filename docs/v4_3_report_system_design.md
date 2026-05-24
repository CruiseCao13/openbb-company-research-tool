# v4.3 Report System Design

v4.3 changes the project from a polished report renderer into an asset-aware research workflow.

The core rule is:

```text
Report shell can be reused. Research logic cannot be blindly reused.
```

## Pipeline

```text
Market and fundamental data
-> deterministic metrics
-> asset profile
-> thesis spine
-> profile-specific report blocks
-> interpretation patch loop
-> lifecycle and company-specificity checks
-> organized report pack
```

## Asset Profiles

The router does not hardcode tickers. It uses public provider fields and calculated metrics:

- sector, industry, and business summary clues
- revenue growth and latest revenue growth
- gross margin, operating margin, and FCF margin
- positive net income and positive FCF years
- PE, PS, and EV / Revenue availability
- cash runway and cash-flow profile
- business-model clues such as financials, cyclicals, biotech-like, aerospace / space systems, and fund-like instruments

Supported first-pass profiles:

- Mature Compounder
- Speculative Growth
- Unprofitable Growth
- Financials
- Cyclical
- Hybrid Growth Compounder
- Unknown / Data-Limited Screening

## Interpretation Patch

Deterministic data is locked. Interpretation blocks are editable.

The patch loop may update:

- one-line verdict
- thesis spine wording
- research battle card
- key questions and answers
- valuation explanation
- next checks

The patch loop may not update:

- market price data
- calculated returns
- valuation snapshot values
- financial statement values
- audit status
- chart paths
- locked scenario assumptions

Patch artifacts are written to `ai/`:

- `correction_patch.json`
- `patched_report_blocks.json`
- `patch_diff_log.md`

## Company Specificity

The company-specificity gate checks whether the selected profile, valuation method, Q&A, red flags, and next checks match the report body.

If the framework coverage is partial or unknown, the report remains usable as screening output, but the run status exposes the limitation.

## Unknown Companies

Unknown companies should not receive a confident template conclusion.

Correct behavior:

- downgrade to Unknown / Data-Limited Screening
- mark framework coverage as `SCREENING_ONLY` or `UNKNOWN`
- generate framework gap analysis
- generate manual verification steps
- keep the report readable while exposing uncertainty


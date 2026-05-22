# Report Structure

Generated reports are designed as first-pass research data packs. They organize public-market data into a repeatable workflow, but they do not replace primary-source research.

## One-line Verdict

A short, plain-English summary of what the data suggests.

It is intentionally not a buy/sell recommendation. It frames the ticker as a research candidate, watchlist idea, data-limited case, or risk-heavy situation.

## Key Takeaways

The most important automatic observations from the run.

Typical signals include:

- Whether the ticker beat or lagged the benchmark
- Whether raw return and risk-adjusted return tell the same story
- Whether drawdown was materially worse than the benchmark
- Whether revenue growth, margin quality, and free cash flow support the narrative
- Whether valuation makes the thesis harder to justify

## Data Confidence

A quick view of how much trust to place in each data area.

Price data is often more usable than company fundamentals from free providers. Financial statements, valuation fields, and ownership data should be verified with filings and company investor relations.

## Automatic Data Warnings

Rules-based warnings for issues that can make analysis misleading.

Examples:

- Target and benchmark use different currencies
- Ownership percentage is above 100%
- PE or EV/EBITDA is negative
- Price history is shorter than roughly one trading year
- Financial statement history has fewer than three usable years
- Revenue or free cash flow data is missing
- ETF / fund-like instruments are being analyzed

No warning does not mean the data is correct. It only means no automatic warning rule was triggered.

## Company Profile

Basic company context from the data provider.

This section is useful for orientation, but business descriptions should be checked against primary sources such as annual reports, 10-K filings, investor presentations, and earnings transcripts.

## Price vs Benchmark

The benchmark comparison is designed to answer a simple opportunity-cost question:

```text
Why not simply hold the benchmark?
```

The report includes three charts:

- Actual Close Price: raw closing prices from the provider
- Normalized Performance: relative performance with the first selected price set to 100
- Drawdown: decline from each asset's prior peak

The price summary table includes total return, CAGR, volatility, max drawdown, Sharpe, Sortino, Calmar, beta, alpha, correlation, tracking error, information ratio, and upside/downside capture.

## Growth and Quality Summary

This section condenses company fundamentals into a few first-pass quality signals:

- Revenue CAGR
- Latest revenue growth
- Gross margin
- Operating margin
- Free cash flow margin
- Positive net income years
- Positive free cash flow years

The goal is to check whether the growth story is supported by economics and cash conversion.

## Money Source and Money Flow

This section shows where money comes from and where it goes.

Key rows include:

- Revenue
- Gross profit
- Operating income
- Net income
- Operating cash flow
- Capital expenditure
- Free cash flow

For ETF / fund-like instruments, this section is skipped because company financial statements are not the right analytical frame.

## Valuation Snapshot

Valuation fields are grouped so they are easier to read:

- Market Size
- Valuation Multiples
- Profitability and Growth
- Cash Flow and Debt
- Ownership
- Price Range

These fields are not enough for valuation. They are a quick screen for whether the market is already pricing in a demanding future.

## Research Potential Score

The Research Potential Score is a heuristic research-priority score.

It is not:

- A valuation model
- A prediction model
- A buy/sell signal
- Investment advice

Score components:

- Growth Score: revenue CAGR and latest revenue growth
- Profitability Score: gross margin, operating margin, and FCF margin
- Quality Trend Score: changes in gross margin, operating margin, and FCF margin
- Risk Control Score: max drawdown, volatility, and beta
- Benchmark Score: excess CAGR, information ratio, and Sharpe difference
- Valuation Sanity Score: penalty-based check using PE, PS, EV/Revenue, and EV/EBITDA

## Required Manual Verification

This section lists items that should be verified manually before forming a serious investment view.

Examples:

- Revenue source and segment breakdown
- Margin trend
- Free cash flow calculation
- Debt and dilution
- Stock-based compensation
- One-time gains or losses
- Management guidance
- SEC 10-K / 10-Q
- Company investor relations materials

## Final Research Questions

The final questions force the user back into judgment.

The tool can help organize evidence, but it cannot answer whether a position belongs in a real portfolio.

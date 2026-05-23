# Report Structure

v2.1 reports are designed as first-pass research workflows. They organize public-market data into a reviewable structure, but they do not replace primary-source research or investment judgment.

The report uses a two-layer reading structure:

- Professional metrics, tables, charts, and calculations
- Plain-English notes that explain what the evidence means for a non-specialist reader

## 0. Boundary

Defines what the report does and does not do.

The report does not provide buy/sell recommendations, price targets, guaranteed returns, trading signals, or portfolio allocation instructions.

## 1. One-line Verdict

A short summary of the current research posture.

It should help the reader understand whether the ticker looks like a mature compounder, a speculative growth company, a data-limited case, a fund-like instrument, or a name with meaningful risk flags.

## 2. How to Read This Report

Explains how to approach the report without understanding every financial term first.

Beginners should start with:

- what the company sells
- whether revenue is growing
- whether revenue becomes cash
- whether the balance sheet is fragile
- whether the stock is already priced for a demanding future

The generated report links to [metric_guide.md](metric_guide.md) for plain-English metric definitions.

## 3. Key Takeaways

Summarizes the main observations from the generated data:

- benchmark outperformance or underperformance
- risk-adjusted performance
- drawdown profile
- revenue growth
- margin quality
- free-cash-flow conversion
- valuation pressure

## 4. Beginner Summary

Translates the main evidence into a simple research map:

- business quality
- growth
- valuation
- balance-sheet risk
- stock risk
- data confidence

This table is not a recommendation. It is a reading aid that helps beginners decide where to look next.

## 5. Data Confidence

Separates confidence by data area:

- price data
- company profile
- financial statements
- valuation snapshot
- segment revenue

Price data is often more usable than provider financial statement data. Segment revenue usually requires manual verification from filings or investor relations materials.

## 6. Sanity Checks

Sanity checks are designed to catch problems that a soft disclaimer would miss.

Each check includes:

- severity
- check name
- finding
- suggested action

Examples:

- currency mismatch
- short price history
- short financial history
- missing revenue
- missing free cash flow
- free-cash-flow consistency issues
- fund-like instrument detected
- elevated balance-sheet or cash-burn risk

## 7. Company Profile

Basic company context from the data provider.

This section is for orientation only. Business descriptions should still be verified against filings, company investor relations pages, earnings releases, and transcripts.

## 8. Price vs Benchmark

The benchmark section is designed around one opportunity-cost question:

```text
Why not simply hold the benchmark?
```

Charts include:

- actual close price
- normalized performance
- drawdown
- interactive HTML dashboard

Metrics include total return, CAGR, volatility, max drawdown, Sharpe, Sortino, Calmar, beta, alpha, correlation, tracking error, information ratio, and capture ratios.

The section includes a plain-English note explaining whether extra return came with extra volatility, weaker risk-adjusted performance, or deeper drawdowns.

## 9. Growth and Quality Summary

Summarizes revenue growth, margin quality, and free-cash-flow conversion.

This section helps answer whether a company is merely growing or whether growth is supported by real business economics.

## 10. Ruin Risk

This section separates normal price volatility from deeper business fragility.

Historical max drawdown is not enough. A company that once fell 30% can still later fall 100% if the business breaks, financing dries up, or dilution becomes destructive.

Metrics include:

- net debt
- EBITDA
- Net Debt / EBITDA
- Debt / FCF
- cash runway
- ruin risk score

## 11. Business Model and Cash Flow

Summarizes money coming into and moving through the business:

- revenue
- gross profit
- operating income
- net income
- operating cash flow
- capital expenditure
- free cash flow

For ETF or fund-like instruments, this section is skipped because company financial statements are not the right analytical frame.

## 12. Personal Margin Stress

Optional account-level stress testing.

If the user passes `--account-equity` and `--margin-loan`, the report estimates portfolio value, margin loan, equity cushion, and loan-to-value under several drawdown scenarios.

This section is intentionally personal. A stock can be analytically interesting and still be dangerous for a leveraged account.

## 13. Valuation Snapshot

Groups valuation and related fields into:

- market size
- valuation multiples
- profitability and growth
- cash flow and debt
- ownership
- price range

These fields are not enough for valuation. They are a first-pass screen for whether the market may already be pricing in a demanding future.

## 14. Research Score

v2.1 uses research profiles instead of one universal score.

Profiles include:

- mature compounder
- profitable growth
- speculative growth
- cyclical / asset-heavy
- financials
- ETF / fund
- data-limited

The score remains a heuristic screening score. It is not a valuation model, prediction model, or investment recommendation.

Generated reports include a beginner warning:

```text
A high score is not a buy signal. A low score is not a sell signal.
```

## 15. Manual Verification

The report explicitly lists what should be checked manually before forming a serious view:

- revenue source and segment breakdown
- gross margin trend
- operating income quality
- free cash flow calculation
- debt and dilution
- stock-based compensation
- one-time gains or losses
- management guidance
- SEC 10-K / 10-Q
- company investor relations materials

## 16. What to Check Next

Turns the report into an action list for further research:

- latest 10-K / 10-Q revenue breakdown
- source of growth
- free-cash-flow quality
- valuation support
- hidden dilution, debt, or margin pressure

## 17. Final Research Questions

The final section forces judgment back to the human.

Examples:

- Why not simply buy the benchmark?
- Has the stock earned its extra risk?
- Is growth real or narrative-driven?
- Is profit quality improving?
- If the stock falls 70%, does the business survive without destructive dilution?
- Is the company being judged against the right lifecycle and sector peers?

# v2 Critical Review Notes

These notes capture the hard critique for the next major improvement cycle. The goal is not to add more surface features, but to make the tool harder to fool, more useful in real research, and less likely to create false confidence.

## 1. Methodology Problems

### Historical Drawdown Is Not Enough

Current reports rely heavily on historical max drawdown, volatility, Sharpe ratio, and benchmark comparison.

Problem:

Historical drawdown can create a false sense of safety. A mature cash-generating company and an unprofitable small-cap growth company can both show a historical drawdown, but their ruin risk is completely different.

Direction:

- Add ruin-risk indicators.
- Consider debt leverage metrics such as Net Debt / EBITDA.
- Consider Altman Z-Score where data quality allows.
- Separate temporary price decline risk from business failure risk.

### One-Size-Fits-All Research Score

Current scoring applies one heuristic model across companies with different lifecycles, sectors, and capital intensity.

Problem:

A model that rewards FCF and margins may punish early-stage growth companies. A model that rewards revenue growth may understate mature compounders. A single score can become a bias filter.

Direction:

- Add sector / lifecycle tags.
- Separate mature compounders, unprofitable growth companies, cyclicals, financials, and ETFs.
- Make scoring weights explicit by category.
- Consider disabling cross-category ranking unless the category is shown clearly.

### Business Model and Cash Flow Is Not Universal

Current financial statement summary works best for standard operating companies with usable yfinance data.

Problem:

Free data sources often have missing or inconsistent segment data. Financials, energy, utilities, and foreign listings may not fit the same operating logic.

Direction:

- Treat segment revenue as manual-first.
- Add sector-specific interpretation notes.
- Avoid implying that the same financial statement logic works equally well across all industries.

## 2. Product and Workflow Problems

### Static PNG Reports Are Low-Interaction

Current output is Markdown plus static PNG charts.

Problem:

Static charts are weak for real research. Users cannot zoom, hover, inspect exact dates, or quickly compare event windows.

Direction:

- Add Plotly interactive HTML charts.
- Consider a local HTML report with embedded interactive charts.
- Longer term: add a minimal Streamlit or browser-based dashboard.

### Setup Script Pollutes Global Shell Environment

Current setup writes a `cresearch` wrapper into `~/.local/bin`.

Problem:

This is convenient but less standard than packaging the tool as an installable CLI. It also stores a fixed project path and can break silently after moving the project.

Direction:

- Add standard package entry points in `pyproject.toml`.
- Consider Typer or Click for CLI ergonomics.
- Support `pip install --editable .`.
- Keep setup script only as an optional convenience path.

### Latest vs Archive Defaults Are Too Easy to Misuse

Current default writes to `latest`, while historical preservation requires `--archive` or `--run-id`.

Problem:

Humans forget. Important snapshots can be overwritten because archiving requires a conscious flag.

Direction:

- Make every run write to timestamped `runs/`.
- Make `latest` a pointer or copy of the newest run.
- Consider daily snapshot retention to avoid excessive clutter.

## 3. Data Reliability Problems

### Automatic Data Warnings Are Too Passive

Current warning language can read like a soft disclaimer when no obvious warning is detected.

Problem:

No warning does not mean data is clean. Free providers can return stale, missing, zero-filled, or internally inconsistent data without triggering obvious exceptions.

Direction:

- Add active sanity checks instead of passive warnings only.
- Compare balance sheet cash change against cash flow statement net cash flow where possible.
- Flag large inconsistencies as high-severity warnings.
- Treat missing core fields as reliability failures, not just empty data.

### Need Financial Statement Consistency Checks

Potential checks:

- Cash change from balance sheet approximately matches net cash flow.
- Free cash flow is consistent with operating cash flow minus capex.
- Revenue, gross profit, and margins do not produce impossible ratios.
- Debt, cash, and enterprise value relationships are plausible.
- Financial years are complete and not silently missing.

## 4. Investment Workflow Problems

### Tool Is Too Generic

The current project is useful as a general research pack generator, but the next version should support a more personal investment workflow.

Direction:

- Add portfolio-context checks.
- Add margin-account stress tests.
- Add excess liquidity stress scenarios.
- Show what happens to user-specific risk if a candidate falls 30%, 50%, or 70%.

## 5. v2 Priority Stack

### P0

- Add Plotly interactive HTML charts.
- Add sector / lifecycle classification.
- Make archive-by-default safer.
- Add active financial statement sanity checks.

### P1

- Add ruin-risk metrics.
- Add category-aware scoring.
- Add high-severity warning levels.
- Add standard installable CLI entry point.

### P2

- Add local dashboard.
- Add portfolio and margin stress testing.
- Add better support for financials, utilities, cyclicals, foreign listings, and ETFs.

## Core Principle

The next version should not merely generate more data.

It should reduce false confidence.

# OpenBB Company Research Tool v3.0

A Python-based company research workflow for turning public market data into structured, archived, and reviewable research reports.

It is built for first-pass company research, not for buy/sell decisions.

v3.0 adds an optional AI Review layer and a cleaner terminal UI. Professional metrics stay deterministic; AI only reviews the generated evidence.

> Data calculates. AI reviews. Human decides.

This tool helps retail investors answer a few basic but important questions before getting emotional about a stock:

- What does the company actually do?
- How has it performed against a benchmark?
- Is the business growing with quality?
- Are there obvious balance-sheet or cash-flow risks?
- Is the data complete enough to trust?
- What should be manually verified before making any judgment?

> This is not a stock-picking machine.  
> It is a research workflow designed to reduce false confidence, messy notes, and emotional decision-making.

---

## 30-Second Demo

```bash
cresearch AAPL --benchmark SPY --start 2023-01-01
```

The tool creates a structured research folder:

```text
reports/AAPL/
├── latest/
│   ├── AAPL_research_report.md
│   ├── AAPL_vs_SPY_interactive_dashboard.html
│   ├── AAPL_vs_SPY_actual_close_price_chart.png
│   ├── AAPL_vs_SPY_performance_chart.png
│   ├── AAPL_vs_SPY_drawdown_chart.png
│   ├── AAPL_research_score_components.png
│   ├── AAPL_growth_quality_trend.png
│   ├── AAPL_ruin_risk_snapshot.png
│   ├── AAPL_sanity_checks.csv
│   ├── AAPL_ruin_risk_snapshot.csv
│   └── AAPL_personal_margin_stress.csv
└── runs/
    └── 20260523_..._AAPL_vs_SPY_start_2023-01-01/
```

Every run is automatically archived under `runs/`.

`latest/` is refreshed as a convenient copy of the newest run, so the user can quickly open the most recent report without losing historical outputs.

---

## Sample Report

- [AAPL sample research report](examples/sample_reports/AAPL_sample_research_report.md)
- [AAPL sample report with mocked AI Review](examples/sample_reports/AAPL_sample_research_report_ai.md)
- [Interactive HTML dashboard](examples/sample_reports/AAPL_vs_SPY_interactive_dashboard.html)

Example excerpt:

```text
Research Profile: Mature Compounder
Research Status: Watchlist

One-line Verdict:
AAPL is a steadily growing, cash-generative name that beat SPY on return,
but the risk-adjusted picture is less clean.

Sanity Checks:
No automatic high-risk consistency failure was detected.
Still verify important numbers with primary sources.
```

The generated report starts with a short "How to Read This Report" section, followed by a Beginner Summary table that translates the main evidence into practical research meaning.

---

## AI Review

v3.0 adds an optional AI Review layer.

The AI Review reads structured report data and checks:

- whether the conclusion is supported by the evidence
- what risks or weak assumptions may be missing
- what a beginner might misunderstand
- what should be manually verified next

It does not provide buy/sell recommendations, price targets, or short-term predictions.

The normal report works without AI.

```bash
export OPENAI_API_KEY="your_openai_api_key_here"
cresearch AAPL --ai-review
```

Model override:

```bash
cresearch AAPL --ai-review --ai-model gpt-4o-mini
OPENAI_MODEL=gpt-4o-mini cresearch AAPL --ai-review
```

If no API key is found, the report still runs and the AI Review section is skipped with a clear reason.

---

## Charts

### Actual Close Price

Raw closing prices for the stock and benchmark.

Useful for checking absolute price levels, gaps, and overall trend shape.

Each generated report includes a short note explaining how to read the chart and what the chart does not prove.

![AAPL vs SPY actual close price](examples/sample_reports/AAPL_vs_SPY_actual_close_price_chart.png)

### Normalized Performance

Both assets start at 100, making relative performance easier to compare.

![AAPL vs SPY normalized performance](examples/sample_reports/AAPL_vs_SPY_normalized_performance_chart.png)

### Drawdown

Shows how far each asset has fallen from its previous peak.

![AAPL vs SPY drawdown](examples/sample_reports/AAPL_vs_SPY_drawdown_chart.png)

### Research Score Components

Breaks down what supports or weakens the research score.

![AAPL research score components](examples/sample_reports/AAPL_research_score_components.png)

### Growth Quality Trend

Tracks revenue growth, margin quality, and free-cash-flow conversion.

![AAPL growth and quality trend](examples/sample_reports/AAPL_growth_quality_trend.png)

### Ruin Risk

Separates normal price volatility from deeper business fragility such as leverage, weak cash flow, or limited cash runway.

![AAPL ruin risk snapshot](examples/sample_reports/AAPL_ruin_risk_snapshot.png)

---

## What v3.0 Improves

| Problem | v3.0 Response |
| --- | --- |
| AI can blur the line between data and opinion | Keeps calculations deterministic and uses AI only as optional review |
| AI output can be hard to test | Uses Chat Completions structured outputs with a Pydantic schema |
| API failures can break workflows | Falls back to a skipped AI Review section without crashing |
| Long-running API calls can feel frozen | Adds Rich terminal status output and an AI review spinner |
| Markdown should not be scraped for AI input | Builds AI payloads from structured `report_data` |

## What v2.1 Improved

| Problem | v2.1 Response |
| --- | --- |
| Finance terms can intimidate beginners | Adds plain-English meaning under major report sections |
| Scores can be mistaken for buy/sell signals | Adds an explicit beginner warning under Research Score |
| Charts can be misread as proof | Adds chart-reading notes explaining what each chart shows and does not show |
| Metric definitions were scattered | Adds [docs/metric_guide.md](docs/metric_guide.md) |
| Reports could still feel like metric dumps | Adds a Beginner Summary table and clearer next-step research prompts |

## What v2.0 Improved

| Problem | v2.0 Response |
| --- | --- |
| Static PNG charts are hard to inspect | Adds Plotly interactive HTML dashboard |
| Historical drawdown can understate business risk | Adds balance-sheet and cash-flow risk checks |
| One-size-fits-all scoring can be misleading | Adds sector/lifecycle-aware scoring weights |
| Data warnings were too passive | Adds sanity checks with severity, finding, and action |
| Users may forget to archive reports | Archives every run by default |
| Reports lacked a clear review workflow | Adds structured report sections and manual verification prompts |
| Generic analysis ignores personal leverage risk | Adds optional margin stress testing |

---

## Core Features

- Benchmark comparison against `SPY`, `VOO`, `QQQ`, or another ticker
- Optional AI Review using OpenAI Chat Completions structured outputs
- Professional Rich-based terminal output with plain-print fallback
- Static PNG charts and Plotly interactive HTML dashboard
- Actual close price, normalized performance, and drawdown views
- Return, volatility, Sharpe, Sortino, Calmar, beta, alpha, tracking error, information ratio, and capture ratios
- Company profile, valuation snapshot, and financial statement summary
- Growth quality and free-cash-flow trend
- Beginner Summary table and plain-English explanations
- Chart-reading notes in generated reports
- Beginner-friendly metric guide
- Balance-sheet and cash-flow risk indicators:
  - Net Debt / EBITDA
  - Debt / FCF
  - Cash runway
  - EBITDA availability
  - Free-cash-flow coverage
- Sanity checks for:
  - missing data
  - short price history
  - currency mismatch
  - free-cash-flow inconsistency
  - fund-like instruments
- Category-aware scoring for:
  - mature compounders
  - speculative growth companies
  - profitable growth companies
  - cyclicals
  - financials
  - ETFs
  - data-limited cases
- Optional personal margin stress test with:
  - `--account-equity`
  - `--margin-loan`
- Optional AI Review controls:
  - `--ai-review`
  - `--ai-model`
  - `--ai-review-depth`
  - `--ai-timeout`
  - `--ai-max-output-tokens`
  - `--no-rich`
- Cross-ticker comparison when multiple symbols are passed

---

## What It Is Not

This project does **not** provide:

- buy or sell recommendations
- price targets
- guaranteed returns
- trading signals
- portfolio allocation instructions
- automated investment decisions

The research score is a heuristic screening score.

It is not a valuation model, prediction model, or investment recommendation.

The AI Review is also not a recommendation. It reviews reasoning quality and missing verification steps using only the generated report data.

---

## Quick Start

```bash
zsh setup_environment.zsh
source ~/.zshrc
cresearch --help
cresearch AAPL --benchmark SPY --start 2023-01-01
```

Manual setup:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

python scripts/company_research_tool.py AAPL --benchmark SPY --start 2023-01-01
```

Optional AI setup:

```bash
cp .env.example .env
export OPENAI_API_KEY="your_openai_api_key_here"
python scripts/company_research_tool.py AAPL --ai-review
```

---

## Usage

```bash
# Basic company research
cresearch AAPL

# Compare multiple tickers
cresearch AAPL TSLA RKLB

# Use a growth-heavy benchmark
cresearch NVDA MSFT --benchmark QQQ

# Compare one stock against another
cresearch TSLA --benchmark AAPL --start 2020-01-01

# Custom risk-free rate
cresearch AAPL --risk-free-rate 0.04

# Optional personal margin stress test
cresearch AAPL --account-equity 100000 --margin-loan 25000

# Optional AI review
cresearch AAPL --ai-review

# Deeper AI review with explicit model
cresearch RKLB --ai-review --ai-review-depth deep --ai-model gpt-4o-mini

# Plain terminal output
cresearch AAPL --no-rich

# Custom run folder
cresearch AAPL --run-id thesis_check_2026_05_23
```

---

## Report Structure

Each report follows a repeatable research workflow:

1. Boundary
2. One-line Verdict
3. How to Read This Report
4. Key Takeaways
5. Beginner Summary
6. Data Confidence and Sanity Checks
7. Company Profile
8. Price vs Benchmark
9. Growth and Quality Summary
10. Ruin Risk
11. Business Model and Cash Flow
12. Personal Margin Stress
13. Valuation Snapshot
14. Research Score
15. AI Review, when requested
16. Manual Verification
17. What to Check Next
18. Final Research Questions

See [docs/report_structure.md](docs/report_structure.md) for the report flow and [docs/metric_guide.md](docs/metric_guide.md) for plain-English metric definitions.

---

## Data Sources

- OpenBB
- OpenBB yfinance provider
- yfinance

Free and public financial data can be delayed, incomplete, inconsistent, or wrong.

For serious decisions, key numbers should be verified with:

- SEC filings
- company investor relations pages
- earnings releases
- official financial statements

---

## Setup Note

`setup_environment.zsh` creates a `cresearch` wrapper pointing to the current project folder.

If you move the project folder, rerun:

```bash
zsh setup_environment.zsh
```

---

## License

MIT License.

# OpenBB Company Research Tool

A command-line company research tool that generates standardized financial research data packs with benchmark comparison, charts, valuation snapshots, data warnings, and Markdown reports.

It is designed for personal investment research workflows: fast enough for screening, structured enough for repeatable analysis, and explicit about data limitations.

## Example Output

Input:

```bash
cresearch AAPL --benchmark SPY --start 2023-01-01
```

Output:

```text
reports/AAPL/latest/
├── AAPL_research_report.md
├── AAPL_vs_SPY_actual_close_price_chart.png
├── AAPL_vs_SPY_performance_chart.png
├── AAPL_vs_SPY_drawdown_chart.png
├── AAPL_vs_SPY_price_summary.csv
├── AAPL_money_source_and_flow.csv
├── AAPL_fundamental_summary.csv
├── AAPL_valuation_snapshot.csv
└── AAPL_research_potential_score.csv
```

Sample report:

- [AAPL sample research report](examples/sample_reports/AAPL_sample_research_report.md)

Sample report excerpt:

```text
One-line Verdict:
AAPL is a steadily growing, cash-generative name that beat SPY on return,
but the risk-adjusted picture is less clean, so the current research status is Research More.

Automatic Data Warnings:
No obvious data-quality warnings were detected automatically.
Still verify important numbers with primary sources before using this report.
```

Example benchmark metrics:

```text
Total Return: 147.89% vs 95.98%
CAGR: 30.77% vs 22.00%
Max Drawdown: -33.43% vs -19.00%
Sharpe Ratio: 1.2103 vs 1.4479
Information Ratio: 0.4646
```

## Sample Charts

### Actual Close Price

Shows raw closing prices from the data provider. Use this to inspect absolute price levels, gaps, and trend shape.

![AAPL vs SPY actual close price](examples/sample_reports/AAPL_vs_SPY_actual_close_price_chart.png)

### Normalized Performance

Sets the first available price in the selected period to 100. Use this to compare relative performance, not absolute stock price. In generated reports this file is named `AAPL_vs_SPY_performance_chart.png`; the sample asset below uses a clearer display name.

![AAPL vs SPY normalized performance](examples/sample_reports/AAPL_vs_SPY_normalized_performance_chart.png)

### Drawdown

Shows the decline from each asset's previous peak. A value of `-20%` means the asset fell 20% from its prior high.

![AAPL vs SPY drawdown](examples/sample_reports/AAPL_vs_SPY_drawdown_chart.png)

## What It Does

- Pulls historical price data
- Compares a ticker with a benchmark such as `SPY`, `VOO`, `QQQ`, or another stock
- Generates actual close price, normalized performance, and drawdown charts
- Calculates return, risk, benchmark, and capture metrics
- Pulls company profile, valuation snapshot, and financial statements where available
- Summarizes revenue growth, margins, profitability, and free cash flow
- Adds data confidence notes and automatic data warnings
- Generates a Markdown research report and CSV outputs
- Generates cross-ticker comparison when multiple tickers are provided
- Handles ETF / fund-like instruments more conservatively by skipping company financial statement analysis

## What It Is Not

This project does not provide:

- Buy / sell recommendations
- Price targets
- Guaranteed returns
- Trading signals
- Portfolio allocation instructions
- Automated investment decisions

The Research Potential Score is a heuristic research-priority score. It is not a valuation model, prediction model, or financial advice.

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

## Usage

```bash
# Basic company research pack
cresearch AAPL

# Multiple tickers ranked together
cresearch AAPL TSLA RKLB

# Use a different benchmark
cresearch NVDA MSFT --benchmark QQQ

# Compare one stock against another
cresearch TSLA --benchmark AAPL --start 2020-01-01

# Custom risk-free rate for risk-adjusted metrics
cresearch AAPL --risk-free-rate 0.04

# Preserve a historical run
cresearch AAPL --archive

# Preserve a run with a custom folder name
cresearch AAPL --run-id test_2023_start
```

## Output Modes

By default, each ticker writes to `latest`, so rerunning the same ticker intentionally refreshes the current report.

```text
reports/
├── AAPL/
│   ├── latest/
│   └── runs/
│       ├── 20260522_1930_AAPL_vs_SPY_start_2023-01-01/
│       └── test_2023_start/
└── _comparison/
    ├── latest/
    └── runs/
```

Use `--archive` or `--run-id` when you want to preserve historical runs.

## Report Design

Each generated report follows a repeatable research workflow:

- One-line Verdict
- Key Takeaways
- Data Confidence
- Automatic Data Warnings
- Company Profile
- Price vs Benchmark
- Growth and Quality Summary
- Money Source and Money Flow
- Valuation Snapshot
- Research Potential Score
- Required Manual Verification
- Final Research Questions

See [docs/report_structure.md](docs/report_structure.md) for details on how to read each section.

## Recommended Workflow

```text
Run cresearch
↓
Open reports/TICKER/latest/TICKER_research_report.md
↓
Review One-line Verdict, Key Takeaways, and Data Warnings
↓
Check actual price, normalized performance, and drawdown charts
↓
Review Money Source and Money Flow
↓
Check valuation and Research Potential Score components
↓
Verify important numbers with SEC filings / company IR
↓
Write your own thesis
```

## Data Sources

- OpenBB
- OpenBB yfinance provider
- yfinance

Free/public financial data can be delayed, incomplete, inconsistent, or wrong. For serious decisions, verify key numbers with SEC filings, company investor relations, earnings releases, and official financial statements.

## Setup Note

`setup_environment.zsh` creates a `cresearch` wrapper that points to the current project folder. If you move the project folder, rerun:

```bash
zsh setup_environment.zsh
```

## License

MIT License.

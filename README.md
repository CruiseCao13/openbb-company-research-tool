# OpenBB Company Research Tool v2.0

**EN:** A bilingual company research radar that turns a ticker into a structured research pack with benchmark comparison, interactive charts, sanity checks, ruin-risk indicators, category-aware scoring, and Markdown reports.

**中文：** 一个中英双语公司研究雷达。输入股票代码后，自动生成基准对比、交互式图表、主动断层扫描、毁灭性风险快照、分类加权评分和 Markdown 研究报告。

> Built for first-pass research, not for buy/sell decisions.  
> 用来做第一层研究，不用来替代投资判断。

## ✦ 30-Second Demo / 30 秒理解

```bash
cresearch AAPL --benchmark SPY --start 2023-01-01
```

The tool creates:

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

Every run is archived. `latest/` is refreshed as a convenient copy of the newest run.

每次运行都会自动归档，`latest/` 只是最新结果的便捷副本，不再靠你记得手动加 `--archive`。

## ✦ Sample Report / 示例报告

- [AAPL sample research report](examples/sample_reports/AAPL_sample_research_report.md)
- [Interactive HTML dashboard](examples/sample_reports/AAPL_vs_SPY_interactive_dashboard.html)

Example excerpt:

```text
Research Profile: Mature Compounder
Research Status: Watchlist

One-line Verdict:
AAPL is a steadily growing, cash-generative name that beat SPY on return,
but the risk-adjusted picture is less clean.

Sanity Scan:
No automatic high-risk consistency failure was detected.
Still verify important numbers with primary sources.
```

## ✦ Charts / 图表展示

### 1. Actual Close Price / 真实收盘价

Raw closing prices. Useful for inspecting absolute price level, gaps, and trend shape.

原始收盘价，用于观察绝对价格、跳空和趋势形态。

![AAPL vs SPY actual close price](examples/sample_reports/AAPL_vs_SPY_actual_close_price_chart.png)

### 2. Normalized Performance / 归一化表现

Starts both assets at 100 so relative performance is comparable.

把起始日价格设为 100，用于比较相对收益，而不是比较绝对股价。

![AAPL vs SPY normalized performance](examples/sample_reports/AAPL_vs_SPY_normalized_performance_chart.png)

### 3. Drawdown / 回撤

Shows decline from each asset's previous peak.

展示资产从前期高点下跌的幅度。

![AAPL vs SPY drawdown](examples/sample_reports/AAPL_vs_SPY_drawdown_chart.png)

### 4. Score Components / 评分组件

Shows what supports or drags the Research Potential Score.

展示研究评分由哪些部分支撑、哪些部分拖累。

![AAPL research score components](examples/sample_reports/AAPL_research_score_components.png)

### 5. Growth and Quality Trend / 增长与质量趋势

Shows revenue growth, margin quality, and free cash flow conversion.

展示营收增长、利润率质量和自由现金流转化。

![AAPL growth and quality trend](examples/sample_reports/AAPL_growth_quality_trend.png)

### 6. Ruin Risk Snapshot / 毁灭性风险快照

Separates price volatility from balance-sheet and cash-burn fragility.

把普通价格波动和资产负债表/烧钱风险分开看。

![AAPL ruin risk snapshot](examples/sample_reports/AAPL_ruin_risk_snapshot.png)

## ✦ What v2.0 Fixes / v2.0 解决什么

| Problem | v2.0 Response |
|---|---|
| Static PNGs are hard to inspect | Adds Plotly interactive HTML dashboard |
| Historical drawdown can understate ruin risk | Adds Ruin Risk Snapshot |
| One-size-fits-all score is biased | Adds sector/lifecycle-aware score weights |
| Data warnings were too passive | Adds Sanity Scan with severity and actions |
| Users forget to archive | Archives every run by default |
| Reports were English-heavy | Makes report surfaces bilingual |
| Tool was too generic | Adds optional personal margin stress testing |

## ✦ Core Features / 核心功能

- Benchmark comparison against `SPY`, `VOO`, `QQQ`, or another ticker
- Static PNG charts plus interactive Plotly HTML dashboard
- Actual close price, normalized performance, and drawdown views
- Return, volatility, Sharpe, Sortino, Calmar, beta, alpha, tracking error, information ratio, capture ratios
- Company profile, valuation snapshot, financial statement summary
- Growth quality and free cash flow trend
- Ruin-risk indicators such as Net Debt / EBITDA, Debt / FCF, and cash runway
- Active sanity checks for missing data, short history, currency mismatch, FCF inconsistency, and fund-like instruments
- Category-aware scoring for mature compounders, speculative growth, profitable growth, cyclicals, financials, ETFs, and data-limited cases
- Optional personal margin stress test with `--account-equity` and `--margin-loan`
- Cross-ticker comparison when multiple symbols are passed

## ✦ What It Is Not / 它不是

This project does **not** provide:

- Buy / sell recommendations
- Price targets
- Guaranteed returns
- Trading signals
- Portfolio allocation instructions
- Automated investment decisions

这个项目不提供买卖建议、目标价、收益承诺、交易信号或仓位建议。

The score is a research-priority heuristic, not a valuation model or prediction model.

评分只是研究优先级启发式模型，不是估值模型或预测模型。

## ✦ Quick Start / 快速开始

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

## ✦ Usage / 用法

```bash
# Basic research radar
cresearch AAPL

# Multiple tickers ranked together
cresearch AAPL TSLA RKLB

# Growth-heavy benchmark
cresearch NVDA MSFT --benchmark QQQ

# Compare one stock against another
cresearch TSLA --benchmark AAPL --start 2020-01-01

# Custom risk-free rate
cresearch AAPL --risk-free-rate 0.04

# Optional personal margin stress
cresearch AAPL --account-equity 100000 --margin-loan 25000

# Custom run folder
cresearch AAPL --run-id thesis_check_2026_05_23
```

## ✦ Report Structure / 报告结构

Each report follows a research workflow:

1. Boundary / 边界
2. One-line Verdict / 一句话判断
3. Key Takeaways / 核心摘要
4. Data Confidence / 数据可信度
5. Sanity Scan / 主动断层扫描
6. Company Profile / 公司资料
7. Price vs Benchmark / 价格与基准比较
8. Ruin Risk Snapshot / 毁灭性风险快照
9. Money Source and Money Flow / 钱从哪里来，流到哪里去
10. Valuation Snapshot / 估值快照
11. Personal Margin Stress / 个人融资压力测试
12. Research Potential Score / 研究潜力评分
13. Manual Verification / 必须人工核对
14. Final Research Questions / 最后必须回答

See [docs/report_structure.md](docs/report_structure.md) for details.

## ✦ Data Sources / 数据源

- OpenBB
- OpenBB yfinance provider
- yfinance

Free/public financial data can be delayed, incomplete, inconsistent, or wrong. For serious decisions, verify key numbers with SEC filings, company investor relations, earnings releases, and official financial statements.

免费公开数据可能延迟、缺失、不一致或错误。严肃判断前必须用 SEC 文件、公司 IR、业绩材料和正式财报核对。

## ✦ Setup Note / 安装说明

`setup_environment.zsh` creates a `cresearch` wrapper pointing to the current project folder. If you move the project folder, rerun:

```bash
zsh setup_environment.zsh
```

## License

MIT License.

# AAPL Research Report｜Company Research Data Pack

> Target: `AAPL`  
> Benchmark: `SPY`  
> Period: `2023-01-01` to `latest available`  
> Research Status: **Research More**  
> Version: `1.3.0`

---

## 0. Boundary｜边界

This report is a standardized research data pack.

It does **not** provide:

- Buy / sell recommendation
- Target price
- Guaranteed return
- Automatic investment decision

The score is a **research prioritization score**, not a prediction.

---

## 1. One-line Verdict｜一句话判断

AAPL is a steadily growing, cash-generative name that beat SPY on return, but the risk-adjusted picture is less clean, so the current research status is Research More.

---

## 2. Key Takeaways｜核心结论摘要

- AAPL beat SPY on both total return and annualized compounding over the selected period.
- Revenue CAGR was 1.81%, which should be checked against the company growth narrative.
- Latest gross margin was 46.91%, useful for judging product/service economics.
- Latest FCF margin was 23.73%, showing how much revenue converts into free cash flow.
- Return leadership came with weaker risk-adjusted efficiency based on Sharpe ratio.
- The stock outperformed, but did so with a deeper drawdown than the benchmark.

---

## 3. Data Confidence｜数据可信度

| Data Area | Confidence | Notes |
|---|---|---|
| Price Data | Medium-High | 850 rows available. Usually usable for historical comparison, but may be delayed or adjusted by provider. |
| Company Profile | Medium | Good for quick context, but business description should be verified with company filings. |
| Financial Statements | Medium | Useful for screening; verify important numbers with 10-K / 10-Q. |
| Valuation Snapshot | Medium | Useful for first-pass valuation risk, not enough for final judgment. |
| Segment Revenue | Manual Required | Usually requires SEC filings or company IR. |


### Automatic Data Warnings

No obvious data-quality warnings were detected automatically. Still verify important numbers with primary sources before using this report.

---

## 4. Company Profile｜公司资料

| Field | Value |
| --- | --- |
| shortName | Apple Inc. |
| longName | Apple Inc. |
| symbol | AAPL |
| quoteType | EQUITY |
| sector | Technology |
| industry | Consumer Electronics |
| country | United States |
| exchange | NMS |
| currency | USD |
| marketCap | 4.55T |
| enterpriseValue | 4.50T |
| beta | 1.0650 |
| website | https://www.apple.com |

### Automatic Business Summary

Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide. The company offers iPhone, a line of smartphones; Mac, a line of personal computers; iPad, a line of multi-purpose tablets; and wearables, home, and accessories comprising AirPods, Apple Vision Pro, Apple TV, Apple Watch, Beats products, and HomePod, as well as Apple branded and third-party accessories. It also provides AppleCare support and cloud services; and operates various platforms, including the App Store that allow customers to discover and download applications and digital content, such as books, music, video, games, and podcasts, as well as advertising services include third-party licensing arrangements and its own advertising platforms. In addition, the company offers various subscription-based services, such as Apple Arcade, a game subscription service; Apple Fitness+, a personalized fitness service; Apple Music, which offers users a curated listening experience with on-demand radio stations; Apple News+, a subscription news and magazine service; Apple TV, which offers exclusive original content and live sports; Apple Card, a co-branded credit card; and Apple Pay, a cashless payment service, as well as licenses its intellectual property. The company serves consumers, and small and mid-sized businesses; and the education, enterprise, and government markets. It distributes third-party applications for its products through the App Store. The c...

### Manual Narrative Needed

You still need to manually answer:

- What does the company actually sell?
- Who pays the company?
- What is the main revenue source?
- What is the growth story?
- Is the story supported by financial data?
- What can break the thesis?

---

## 5. Price vs Benchmark｜价格与基准比较

Benchmark explanation:

> SPY is a broad S&P 500 benchmark. It tests whether the stock deserves capital compared with a simple broad-market ETF.

![AAPL vs SPY Actual Close Price](AAPL_vs_SPY_actual_close_price_chart.png)

Actual close price chart shows the raw closing prices from the data provider.
Use this to inspect absolute price levels, gaps, and broad trend shape before comparing relative returns.

真实收盘价图显示数据源返回的原始收盘价，用于查看绝对价格水平、价格缺口和整体走势。

![AAPL vs SPY](AAPL_vs_SPY_normalized_performance_chart.png)

Performance chart uses normalized price.
The first available price in the selected period is set to 100.
This allows comparison of relative performance, not absolute stock price.

该图使用归一化价格。起始日价格被设为 100，用于比较相对收益，而不是显示真实股价。

![AAPL vs SPY Drawdown](AAPL_vs_SPY_drawdown_chart.png)

Drawdown shows the decline from the previous peak.
0% means no drawdown.
-20% means the asset fell 20% from its previous high.

回撤图表示资产从此前高点下跌的幅度。0% 表示没有回撤，-20% 表示从高点下跌 20%。

| Metric | Target | Benchmark | Difference |
| --- | --- | --- | --- |
| Total Return | 147.89% | 95.98% | 51.91% |
| CAGR | 30.77% | 22.00% | 8.77% |
| 1Y Return | 53.97% | 28.00% | 25.97% |
| 6M Return | 16.45% | 14.37% | 2.07% |
| 3M Return | 13.92% | 8.58% | 5.34% |
| Max Drawdown | -33.43% | -19.00% | -14.43% |
| Annualized Volatility | 25.43% | 15.19% | 10.23% |
| Sharpe Ratio | 1.2103 | 1.4479 | -0.2376 |
| Sortino Ratio | 1.7397 | 1.9756 | -0.2359 |
| Calmar Ratio | 0.9204 | 1.1580 | -0.2375 |
| Beta vs Benchmark | 1.1278 | N/A | N/A |
| Alpha vs Benchmark | 6.34% | N/A | N/A |
| Correlation vs Benchmark | 0.6740 | N/A | N/A |
| Tracking Error | 18.88% | N/A | N/A |
| Information Ratio | 0.4646 | N/A | N/A |
| Upside Capture | 109.27% | N/A | N/A |
| Downside Capture | 99.32% | N/A | N/A |

### How to Read This

- **Total Return / CAGR**: raw performance.
- **Excess Return**: whether the stock outperformed the benchmark.
- **Max Drawdown**: deepest historical decline in this period.
- **Sharpe / Sortino / Calmar**: risk-adjusted performance.
- **Beta**: sensitivity to benchmark movement.
- **Information Ratio**: excess return per unit of tracking risk.
- **Upside / Downside Capture**: whether the stock captures more upside or downside than benchmark.

---

## 6. Growth and Quality Summary｜增长与质量比较

| Metric | Value |
| --- | --- |
| Revenue CAGR | 1.81% |
| Revenue Growth Latest | 6.43% |
| Gross Margin Latest | 46.91% |
| Gross Margin Change | 3.60% |
| Operating Margin Latest | 31.97% |
| Operating Margin Change | 1.68% |
| FCF Margin Latest | 23.73% |
| FCF Margin Change | -4.53% |
| Positive Net Income Years | 4 |
| Positive FCF Years | 4 |

### Core Questions

- Is revenue growing?
- Is growth accelerating or slowing?
- Is gross margin stable or improving?
- Is operating margin improving?
- Is free cash flow improving?

---

## 7. Money Source and Money Flow｜钱从哪里来，流到哪里去

| index | Revenue | Gross Profit | Operating Income | Net Income | Operating Cash Flow | Capital Expenditure | Free Cash Flow | Revenue Growth YoY | Gross Margin | Operating Margin | Net Margin | FCF Margin |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2022-09-30 | 394.33B | 170.78B | 119.44B | 99.80B | 122.15B | -10.71B | 111.44B | N/A | 43.31% | 30.29% | 25.31% | 28.26% |
| 2023-09-30 | 383.29B | 169.15B | 114.30B | 97.00B | 110.54B | -10.96B | 99.58B | -2.80% | 44.13% | 29.82% | 25.31% | 25.98% |
| 2024-09-30 | 391.04B | 180.68B | 123.22B | 93.74B | 118.25B | -9.45B | 108.81B | 2.02% | 46.21% | 31.51% | 23.97% | 27.83% |
| 2025-09-30 | 416.16B | 195.20B | 133.05B | 112.01B | 111.48B | -12.71B | 98.77B | 6.43% | 46.91% | 31.97% | 26.92% | 23.73% |

### Interpretation

- Revenue shows money coming in.
- Gross profit shows whether product/service economics work.
- Operating income shows whether the operating model works.
- Net income shows accounting profit.
- Operating cash flow shows whether business operations generate cash.
- Free cash flow shows whether cash remains after capital expenditure.

---

## 8. Valuation Snapshot｜估值快照

### Market Size

| Metric | Value |
| --- | --- |
| marketCap | 4.55T |
| enterpriseValue | 4.50T |

### Valuation Multiples

| Metric | Value |
| --- | --- |
| trailingPE | 37.4958 |
| forwardPE | 32.2882 |
| priceToSalesTrailing12Months | 10.0873 |
| priceToBook | 42.7121 |
| enterpriseToRevenue | 9.9590 |
| enterpriseToEbitda | 28.1020 |

### Profitability and Growth

| Metric | Value |
| --- | --- |
| grossMargins | 47.86% |
| operatingMargins | 32.27% |
| profitMargins | 27.15% |
| returnOnEquity | 141.47% |
| returnOnAssets | 26.23% |
| revenueGrowth | 16.60% |
| earningsGrowth | 21.80% |

### Cash Flow and Debt

| Metric | Value |
| --- | --- |
| operatingCashflow | 140.22B |
| freeCashflow | 101.09B |
| totalCash | 68.51B |
| totalDebt | 84.71B |

### Ownership

| Metric | Value |
| --- | --- |
| sharesOutstanding | 14,687,356,000 |
| floatShares | 14,662,387,495 |
| heldPercentInsiders | 1.63% |
| heldPercentInstitutions | 65.96% |

### Price Range

| Metric | Value |
| --- | --- |
| fiftyTwoWeekLow | 193.4600 |
| fiftyTwoWeekHigh | 310.0900 |

High valuation requires stronger growth, margin expansion, and cash flow evidence.

---

## 9. Research Potential Score｜研究潜力评分

| Component | Score | Weight |
| --- | --- | --- |
| Growth Score | 40.75 / 100 | 22.00% |
| Profitability Score | 85.94 / 100 | 22.00% |
| Quality Trend Score | 50.75 / 100 | 16.00% |
| Risk Control Score | 65.42 / 100 | 16.00% |
| Benchmark Score | 58.41 / 100 | 16.00% |
| Valuation Sanity Score | 31.78 / 100 | 8.00% |
| Research Potential Score | 58.35 / 100 | 100.00% |

### Why This Score?

This score is a heuristic research-priority score.
It is not a valuation model, not a prediction model, and not a buy/sell signal.

- Growth Score: based on revenue CAGR and latest revenue growth.
- Profitability Score: based on gross margin, operating margin, and FCF margin.
- Quality Trend Score: based on changes in gross margin, operating margin, and FCF margin.
- Risk Control Score: based on max drawdown, volatility, and beta.
- Benchmark Score: based on excess CAGR, information ratio, and Sharpe difference.
- Valuation Sanity Score: penalty-based score using PE, PS, EV/Revenue, and EV/EBITDA.

Main score support:
- Profitability Score: 85.94 / 100
- Risk Control Score: 65.42 / 100

Main score drag:
- Valuation Sanity Score: 31.78 / 100
- Growth Score: 40.75 / 100

### Score Meaning

- 75–100: High Priority Research
- 60–75: Watchlist
- 45–60: Research More
- 30–45: FOMO Risk / Weak Evidence
- 0–30: Avoid for Now / Data Weak

This score is transparent but imperfect. It is used to prioritize research, not to make investment decisions.

---

## 10. Required Manual Verification｜必须人工核对

Before making any serious judgment, verify:

- Revenue source and segment breakdown
- Gross margin trend
- Operating income quality
- Free cash flow calculation
- Debt and dilution
- Stock-based compensation
- One-time gains/losses
- Management guidance
- SEC 10-K / 10-Q
- Company IR materials

---

## 11. Final Research Questions｜最后必须回答

- Why not simply buy `SPY`?
- Has `AAPL` earned its extra risk?
- Is growth real or narrative-driven?
- Is profit quality improving?
- Is free cash flow healthy?
- Is valuation already pricing in too much future success?
- If the stock falls 30%-50%, does the thesis still hold?

---

## 12. Generated Files

This folder contains CSV, chart, and Markdown outputs generated by the tool.

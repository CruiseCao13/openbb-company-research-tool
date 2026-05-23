# AAPL Research Report｜Company Research Radar v2.0

> Target: `AAPL`  
> Benchmark: `SPY`  
> Period: `2023-01-01` to `latest available`  
> Research Status: **Watchlist**  
> Research Profile: **Mature Compounder**  
> Version: `2.0.0`

---

## 0. Boundary｜边界

This report is a bilingual research radar.

It does **not** provide:

- Buy / sell recommendation
- Target price
- Guaranteed return
- Automatic investment decision

The score is a **research prioritization score**, not a prediction.

本报告是研究雷达，不是投资建议。它的目标是暴露问题、组织证据、降低自我安慰，而不是替你做决定。

---

## 1. One-line Verdict｜一句话判断

AAPL is a steadily growing, cash-generative name that beat SPY on return, but the risk-adjusted picture is less clean, so the current research status is Watchlist.

---

## 2. Key Takeaways｜核心结论摘要

- AAPL beat SPY on both total return and annualized compounding over the selected period.
- Return leadership came with weaker risk-adjusted efficiency based on Sharpe ratio.
- The stock outperformed, but did so with a deeper drawdown than the benchmark.
- Revenue CAGR was 1.81%, which should be checked against the company growth narrative.
- Latest gross margin was 46.91%, useful for judging product/service economics.
- Latest FCF margin was 23.73%, showing how much revenue converts into free cash flow.
- Valuation sanity score is weak, so the thesis requires stronger growth and cash-flow evidence.
- Research Potential Score is 61.34 / 100, classified as Watchlist.

---

## 3. Data Confidence｜数据可信度

| Data Area | Confidence | Notes |
|---|---|---|
| Price Data | Medium-High | 850 rows available. Usually usable for historical comparison, but may be delayed or adjusted by provider. |
| Company Profile | Medium | Good for quick context, but business description should be verified with company filings. |
| Financial Statements | Medium | Useful for screening; verify important numbers with 10-K / 10-Q. |
| Valuation Snapshot | Medium | Useful for first-pass valuation risk, not enough for final judgment. |
| Segment Revenue | Manual Required | Usually requires SEC filings or company IR. |


### 🧯 Sanity Scan｜主动断层扫描

| Severity | Check | Finding | Action |
| --- | --- | --- | --- |
| INFO | No triggered sanity failure / 未触发重大断层 | No automatic high-risk consistency failure was detected. | Still verify important numbers with primary sources. / 仍需用一手资料核对关键数字。 |

### Automatic Data Warnings

No legacy warning rule was triggered. See the Sanity Scan below for stronger consistency checks.

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
| marketCap | 4.54T |
| enterpriseValue | 4.55T |
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

### 🕹 Interactive HTML｜交互式图表

[Open interactive price dashboard](AAPL_vs_SPY_interactive_dashboard.html)

The HTML chart supports hover, zoom, range selection, and exact-date inspection.

交互式 HTML 图表支持悬停、缩放、区间选择和按日期查看具体数值。

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
| Total Return | 146.92% | 95.80% | 51.12% |
| CAGR | 30.62% | 21.97% | 8.66% |
| 1Y Return | 53.37% | 27.88% | 25.49% |
| 6M Return | 15.99% | 14.27% | 1.72% |
| 3M Return | 13.48% | 8.48% | 5.00% |
| Max Drawdown | -33.43% | -19.00% | -14.43% |
| Annualized Volatility | 25.42% | 15.19% | 10.23% |
| Sharpe Ratio | 1.2046 | 1.4458 | -0.2411 |
| Sortino Ratio | 1.7311 | 1.9726 | -0.2415 |
| Calmar Ratio | 0.9159 | 1.1562 | -0.2403 |
| Beta vs Benchmark | 1.1275 | N/A | N/A |
| Alpha vs Benchmark | 6.26% | N/A | N/A |
| Correlation vs Benchmark | 0.6739 | N/A | N/A |
| Tracking Error | 18.88% | N/A | N/A |
| Information Ratio | 0.4584 | N/A | N/A |
| Upside Capture | 109.17% | N/A | N/A |
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

## 7. Ruin Risk Snapshot｜毁灭性风险快照

![AAPL Ruin Risk Snapshot](AAPL_ruin_risk_snapshot.png)

| Metric | Value | Interpretation |
| --- | --- | --- |
| Net Debt | 16.20B | Total debt minus cash. Negative is net cash. |
| EBITDA | 159.98B | Provider EBITDA, when available. |
| Net Debt / EBITDA | 0.1013 | Debt-load proxy. Higher values deserve manual stress testing. |
| Debt / FCF | 0.8380 | Debt compared with free cash flow. Not useful when FCF is negative. |
| Cash Runway Years | N/A | Approximate years of cash runway when FCF is negative. |
| Ruin Risk Score | 51.22 / 100 | N/A |

This section tries to separate normal price volatility from business fragility. Historical drawdown is not the same as ruin risk.

这一节用于区分“价格波动”和“业务毁灭性风险”。历史回撤不等于破产、融资枯竭或商业模式失效风险。

---

## 8. Money Source and Money Flow｜钱从哪里来，流到哪里去

![AAPL Growth and Quality Trend](AAPL_growth_quality_trend.png)

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

## 9. Personal Margin Stress｜个人融资压力测试

| Scenario | Portfolio Value | Margin Loan | Equity Cushion | Loan / Value |
| --- | --- | --- | --- | --- |
| -20.00% portfolio shock | 80,000.00 | 25,000.00 | 55,000.00 | 31.25% |
| -30.00% portfolio shock | 70,000.00 | 25,000.00 | 45,000.00 | 35.71% |
| -50.00% portfolio shock | 50,000.00 | 25,000.00 | 25,000.00 | 50.00% |
| -70.00% portfolio shock | 30,000.00 | 25,000.00 | 5,000.00 | 83.33% |

This optional section is not about the company. It tests whether your own balance sheet can survive stress.

这一节不是分析公司，而是检查你自己的资产负债表能不能扛住压力。

---

## 10. Valuation Snapshot｜估值快照

### Market Size

| Metric | Value |
| --- | --- |
| marketCap | 4.54T |
| enterpriseValue | 4.55T |

### Valuation Multiples

| Metric | Value |
| --- | --- |
| trailingPE | 37.4327 |
| forwardPE | 32.1559 |
| priceToSalesTrailing12Months | 10.0472 |
| priceToBook | 42.5372 |
| enterpriseToRevenue | 10.0830 |
| enterpriseToEbitda | 28.4540 |

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
| fiftyTwoWeekLow | 195.0700 |
| fiftyTwoWeekHigh | 311.4000 |

High valuation requires stronger growth, margin expansion, and cash flow evidence.

---

## 11. Research Potential Score｜研究潜力评分

[Open interactive score radar](AAPL_research_score_radar.html)

![AAPL Research Score Components](AAPL_research_score_components.png)

| Component | Score | Weight | Profile |
| --- | --- | --- | --- |
| Growth Score | 40.75 / 100 | 14.00% | Mature Compounder |
| Profitability Score | 85.94 / 100 | 28.00% | Mature Compounder |
| Quality Trend Score | 50.75 / 100 | 18.00% | Mature Compounder |
| Risk Control Score | 65.43 / 100 | 18.00% | Mature Compounder |
| Benchmark Score | 58.22 / 100 | 14.00% | Mature Compounder |
| Valuation Sanity Score | 31.37 / 100 | 8.00% | Mature Compounder |
| Research Potential Score | 61.34 / 100 | 100.00% | Mature Compounder |

### Why This Score?

This score is a heuristic research-priority score, now weighted by research profile.
It is not a valuation model, not a prediction model, and not a buy/sell signal.

这个分数是按研究类型加权的启发式“研究优先级”分数，不是估值模型、预测模型或买卖信号。

- Growth Score / 增长: revenue CAGR and latest revenue growth.
- Profitability Score / 盈利质量: gross margin, operating margin, and FCF margin.
- Quality Trend Score / 质量趋势: changes in gross margin, operating margin, and FCF margin.
- Risk Control Score / 风险控制: max drawdown, volatility, and beta.
- Benchmark Score / 基准对比: excess CAGR, information ratio, and Sharpe difference.
- Valuation Sanity Score / 估值理性: penalty-based check using PE, PS, EV/Revenue, and EV/EBITDA.

Main score support:
- Profitability Score: 85.94 / 100
- Risk Control Score: 65.43 / 100

Main score drag:
- Valuation Sanity Score: 31.37 / 100
- Growth Score: 40.75 / 100

### Score Meaning

- 75–100: High Priority Research
- 60–75: Watchlist
- 45–60: Research More
- 30–45: FOMO Risk / Weak Evidence
- 0–30: Avoid for Now / Data Weak

This score is transparent but imperfect. It is used to prioritize research, not to make investment decisions.

---

## 12. Required Manual Verification｜必须人工核对

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
- Sanity Scan HIGH severity items
- Ruin Risk Snapshot debt and cash-burn assumptions

---

## 13. Final Research Questions｜最后必须回答

- Why not simply buy `SPY`?
- Has `AAPL` earned its extra risk?
- Is growth real or narrative-driven?
- Is profit quality improving?
- Is free cash flow healthy?
- Is valuation already pricing in too much future success?
- If the stock falls 30%-50%, does the thesis still hold?
- If the stock falls 70%, does the business survive without destructive dilution?
- Is this company being judged against the right lifecycle and sector peers?

---

## 14. Generated Files

This folder contains CSV, chart, and Markdown outputs generated by the tool.

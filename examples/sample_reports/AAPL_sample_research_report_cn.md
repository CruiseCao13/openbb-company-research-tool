# AAPL 股票研究报告

> 标的：`AAPL`  
> 基准：`SPY`  
> 期间：`2023-01-01` 到 `最新可得数据`  
> 研究状态：**Watchlist**  
> 研究画像：**Mature Compounder**  
> 版本：`4.0.0`  
> DATA_AUDIT_STATUS：**WARNING**  
> RISK_METHOD_STATUS：**PASS**  
> AI_ANALYST_REVIEW_STATUS：**WARNING**  
> LANGUAGE_LINT_STATUS：**PASS**  
> OVERALL_REPORT_STATUS：**WARNING**  
> 数据审计状态：**WARNING**  
> 语言检查状态：**PASS**  
> Price Label Check：**PASS**

---

## 边界

这份报告用于股票初筛和研究路径生成，不提供买卖建议、目标价、收益承诺或短期预测。

## 核心主线

AAPL 的核心问题不是“公司能不能赚钱”，而是当前价格是否已经把未来的利润率、现金流和增长韧性提前交易进去。

## 投研博弈卡片

### 买入的核心赌注

AAPL 不是纯高增长故事。核心赌注是利润率、现金流、生态粘性和回购能力能继续支撑 EPS。只要这些支柱不塌，市场就可能继续接受高估值。

### 做空或离场的死穴

反方逻辑从增长放慢但估值不降开始。如果服务业务失速，或者毛利率下滑，当前估值缺少缓冲。

### 市场已经交易了什么

当前估值已经在交易高质量现金流和长期韧性。PE 约 37.4327，PS 约 10.0472，这意味着市场不只是在买当期收入增长。

### 什么必须守住

- 毛利率不能持续下滑。
- 自由现金流率要保持健康。
- 核心业务不能出现结构性失速。
- 回购对 EPS 的支撑不能失效。
- 监管不能打穿关键利润池。

### 一票否决条件

- 服务业务增速明显放缓。
- 毛利率连续两个季度下滑。
- 核心收入下滑且没有服务业务接住。
- 高估值削弱回购对 EPS 的支撑。
- 监管压力伤害高利润业务。

### 最优先核查的 3 件事

1. 分业务线收入和服务业务增长。
2. 自由现金流是否稳定，还是受一次性项目影响。
3. 当前估值是否已经透支未来 EPS 增长。


## 初学者摘要

| Area | Status | Plain-English Meaning |
| --- | --- | --- |
| Business Quality | Strong | The company appears cash-generative and profitable. |
| Growth | Moderate | Revenue growth is 1.81% in this data window. |
| Valuation | Expensive | The stock needs strong future execution to justify the current multiple. |
| Balance Sheet Risk | Low | Debt and cash-flow fragility do not appear to be the main first-pass risk. |
| Stock Risk | Medium | The stock can still have painful drawdowns even when the business is strong. |
| Data Confidence | Medium | Good enough for screening, but important numbers still need primary-source verification. |

## 价格与基准

![AAPL vs SPY 实际收盘价](AAPL_vs_SPY_actual_close_price_chart.png)

![AAPL vs SPY 归一化表现](AAPL_vs_SPY_normalized_performance_chart.png)

![AAPL vs SPY 回撤](AAPL_vs_SPY_drawdown_chart.png)

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
| Beta vs Benchmark | 1.1275 | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |
| Alpha vs Benchmark | 6.26% | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |
| Correlation vs Benchmark | 0.6739 | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |
| Tracking Error | 18.88% | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |
| Information Ratio | 0.4584 | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |
| Upside Capture | 109.17% | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |
| Downside Capture | 99.32% | [METRIC_MISSING_RAW] | [METRIC_MISSING_RAW] |

## 风险指标方法

RISK_METHOD_STATUS: PASS

- 价格字段：`adj_close`
- 收益率频率：daily
- 年化天数：252
- 无风险利率：0.00%
- 基准：SPY
- 缺失值处理：对齐交易日，删除目标和基准任一缺失的行。
- 股息处理：股息是否进入收益率，取决于数据源的复权价格口径。

风险指标使用日频复权收盘价、252 个交易日年化、0.00% 无风险利率。股息是否进入收益率，取决于数据源的复权价格口径。


## 成长与质量

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

## 资产负债表韧性

![AAPL 资产负债表韧性](AAPL_ruin_risk_snapshot.png)

资产负债表韧性分数：分数越高，财务韧性越强。这个分数衡量的是资产负债表韧性，不是股价波动风险。

| Metric | Value | Interpretation |
| --- | --- | --- |
| Net Debt | 16.20B | Total debt minus cash. Negative is net cash. |
| EBITDA | 159.98B | Provider EBITDA, when available. |
| Net Debt / EBITDA | 0.1013 | Debt-load proxy. Higher values deserve manual stress testing. |
| Debt / FCF | 0.8380 | Debt compared with free cash flow. Not useful when FCF is negative. |
| Cash Runway Years | [METRIC_MISSING_RAW] | Approximate years of cash runway when FCF is negative. |
| Balance Sheet Resilience Score | 48.78 / 100 | [METRIC_MISSING_RAW] |

## 业务线拆解

这家公司必须拆业务线。只看总收入，会把真正的增长来源和利润结构藏起来。

需要手工核查：

- iPhone / Mac / iPad / Wearables / Services，或对应公司的主要业务线；
- 各业务线 YoY 增长；
- 各业务线收入占比；
- 高毛利业务是否在提升利润质量；
- 监管是否影响高利润业务。


## 估值快照

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
| heldPercentInstitutions | 65.80% |

### Price Range

| Metric | Value |
| --- | --- |
| fiftyTwoWeekLow | 195.0700 |
| fiftyTwoWeekHigh | 311.4000 |

## 估值压力测试

这不是目标价预测，而是估值压力测试。它回答的不是“应该值多少钱”，而是“如果市场开始杀估值，可能有多疼”。

| 场景 | 对比 | EPS 增长假设 |
|---|---|---|
| PE 压缩到 30x | 与当前 37.4327x 比较 | EPS 增长假设：0%, 5%, 10%, 15% |
| PE 压缩到 25x | 与当前 37.4327x 比较 | EPS 增长假设：0%, 5%, 10%, 15% |
| PE 压缩到 20x | 与当前 37.4327x 比较 | EPS 增长假设：0%, 5%, 10%, 15% |


## Research Score

| Component | Score | Weight | Profile |
| --- | --- | --- | --- |
| Growth Score | 40.75 / 100 | 14.00% | Mature Compounder |
| Profitability Score | 85.94 / 100 | 28.00% | Mature Compounder |
| Quality Trend Score | 50.75 / 100 | 18.00% | Mature Compounder |
| Risk Control Score | 65.43 / 100 | 18.00% | Mature Compounder |
| Benchmark Score | 58.22 / 100 | 14.00% | Mature Compounder |
| Valuation Sanity Score | 31.37 / 100 | 8.00% | Mature Compounder |
| Research Score | 61.34 / 100 | 100.00% | Mature Compounder |

Research Score 是研究优先级分数，不是预期收益、不是安全边际、不是买卖信号。

## 最优先核查的 3 件事

1. 分业务线收入和服务业务增长。
2. 自由现金流是否稳定，还是受一次性项目影响。
3. 当前估值是否已经透支未来 EPS 增长。

## 结论边界

当前分数为 61.34 / 100。它只说明这家公司值得按上述路径继续核查，不代表应该买入或卖出。

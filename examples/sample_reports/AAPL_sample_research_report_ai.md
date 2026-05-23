# AAPL Research Report

> Target: `AAPL`  
> Benchmark: `SPY`  
> Period: `2023-01-01` to `latest available`  
> Research Status: **Watchlist**  
> Research Profile: **Mature Compounder**  
> Version: `3.0.0`

---

## 0. Boundary

This report is a structured first-pass research workflow.

It does **not** provide:

- Buy / sell recommendation
- Target price
- Guaranteed return
- Automatic investment decision

The score is a **research prioritization score**, not a prediction.

---

## 1. One-line Verdict

AAPL looks like a mature cash-flow business rather than a high-growth story; it beat SPY on return, but the risk-adjusted picture is less clean, so the current research status is Watchlist. Valuation is the main constraint on the first-pass setup.

---

## 2. How to Read This Report

This report is designed for first-pass research.

You do not need to understand every financial term at the beginning. Start with five questions:

1. What does the company sell?
2. Is revenue growing?
3. Does the company turn revenue into real cash?
4. Is the balance sheet fragile?
5. Is the stock already priced for perfection?

If a metric looks unfamiliar, read the plain-English note below each section first, then use the table as evidence.

Metric guide: [docs/metric_guide.md](../../docs/metric_guide.md)

---

## 3. Key Takeaways

- AAPL beat SPY, but the outperformance was not supported by fast revenue growth; the thesis depends more on margins, cash flow, buybacks, and market willingness to keep paying a premium multiple.
- The stock delivered better raw return, but weaker Sharpe efficiency means the investor was paid less cleanly for each unit of volatility.
- The outperformance came with deeper drawdowns, so position sizing and holding-period discipline matter more than the headline return suggests.
- The business profile is Mature Compounder: revenue CAGR is 1.81%, gross margin is 46.91%, and FCF margin is 23.73%. That points to cash-flow quality, not just top-line momentum.
- Valuation is the main friction point (PE around 37.4x, PS around 10.0x); future returns depend on the company maintaining margin quality and the market continuing to accept a premium multiple.
- The research score is 61.34 / 100 (Watchlist), which should be read as a triage label rather than a conclusion.

---

## 4. Beginner Summary

| Area | Status | Plain-English Meaning |
| --- | --- | --- |
| Business Quality | Strong | The company appears cash-generative and profitable. |
| Growth | Moderate | Revenue growth is 1.81% in this data window. |
| Valuation | Expensive | The stock needs strong future execution to justify the current multiple. |
| Balance Sheet Risk | Medium | Debt and cash-flow fragility do not appear to be the main first-pass risk. |
| Stock Risk | Medium | The stock can still have painful drawdowns even when the business is strong. |
| Data Confidence | Medium | Good enough for screening, but important numbers still need primary-source verification. |

---

## 5. Data Confidence

| Data Area | Confidence | Notes |
|---|---|---|
| Price Data | Medium-High | 850 rows available. Usually usable for historical comparison, but may be delayed or adjusted by provider. |
| Company Profile | Medium | Good for quick context, but business description should be verified with company filings. |
| Financial Statements | Medium | Useful for screening; verify important numbers with 10-K / 10-Q. |
| Valuation Snapshot | Medium | Useful for first-pass valuation risk, not enough for final judgment. |
| Segment Revenue | Manual Required | Usually requires SEC filings or company IR. |


### Sanity Checks

| Severity | Check | Finding | Action |
| --- | --- | --- | --- |
| INFO | No triggered sanity failure | No automatic high-risk consistency failure was detected. | Still verify important numbers with primary sources. |

### Automatic Data Warnings

No additional warning rule was triggered. Review the sanity checks above before relying on the data.

---

## 6. Company Profile

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

## 7. Price vs Benchmark

Benchmark explanation:

> SPY is a broad S&P 500 benchmark. It tests whether the stock deserves capital compared with a simple broad-market ETF.

### Interactive HTML

[Open interactive price dashboard](AAPL_vs_SPY_interactive_dashboard.html)

The HTML chart supports hover, zoom, range selection, and exact-date inspection.

![AAPL vs SPY Actual Close Price](AAPL_vs_SPY_actual_close_price_chart.png)

Actual close price chart shows the raw closing prices from the data provider.
Use this to inspect absolute price levels, gaps, and broad trend shape before comparing relative returns.

How to read this chart: it shows price level, not valuation. A higher line does not mean the stock is cheaper or safer.

![AAPL vs SPY](AAPL_vs_SPY_normalized_performance_chart.png)

Performance chart uses normalized price.
The first available price in the selected period is set to 100.
This allows comparison of relative performance, not absolute stock price.

How to read this chart: both lines start at 100. If one line ends higher, it performed better during this period. This does not prove it is a better investment today.

![AAPL vs SPY Drawdown](AAPL_vs_SPY_drawdown_chart.png)

Drawdown shows the decline from the previous peak.
0% means no drawdown.
-20% means the asset fell 20% from its previous high.

How to read this chart: drawdown shows pain. A -30% drawdown means an investor buying near the previous peak would have seen the position fall by about 30%.

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

### Plain-English Meaning

AAPL made more money than SPY during this period. Its risk-adjusted return was weaker, which means the extra return was not as efficient as it first appears. It also had a deeper drawdown, so investors had to tolerate more pain along the way. The key question is not just which line went up more, but whether the extra return was worth the extra volatility and drawdown.

---

## 8. Growth and Quality Summary

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

### Plain-English Meaning

Growth is not the main story here. Revenue CAGR is 1.81%, so the case depends more on profitability, cash flow, capital returns, and valuation discipline.

---

## 9. Ruin Risk

![AAPL Ruin Risk](AAPL_ruin_risk_snapshot.png)

How to read this chart: this is not about day-to-day stock movement. It asks whether the business could face financial stress if growth slows, cash flow weakens, or refinancing becomes difficult.

| Metric | Value | Interpretation |
| --- | --- | --- |
| Net Debt | 16.20B | Total debt minus cash. Negative is net cash. |
| EBITDA | 159.98B | Provider EBITDA, when available. |
| Net Debt / EBITDA | 0.1013 | Debt-load proxy. Higher values deserve manual stress testing. |
| Debt / FCF | 0.8380 | Debt compared with free cash flow. Not useful when FCF is negative. |
| Cash Runway Years | N/A | Approximate years of cash runway when FCF is negative. |
| Ruin Risk Score | 51.22 / 100 | N/A |

This section tries to separate normal price volatility from business fragility. Historical drawdown is not the same as ruin risk.

### Plain-English Meaning

This section is not about daily stock movement. It asks whether the business could face serious financial stress if growth slows, cash flow weakens, or refinancing becomes difficult.

---

## 10. Business Model and Cash Flow

![AAPL Growth and Quality Trend](AAPL_growth_quality_trend.png)

How to read this chart: rising revenue is useful, but the important question is whether the company keeps enough cash after costs, operations, and capital spending.

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

### Plain-English Meaning

The company generated positive free cash flow in the latest available period. That cash can support reinvestment, buybacks, debt reduction, or dividends.

---

## 11. Personal Margin Stress

| Scenario | Portfolio Value | Margin Loan | Equity Cushion | Loan / Value |
| --- | --- | --- | --- | --- |
| -20.00% portfolio shock | 80,000.00 | 25,000.00 | 55,000.00 | 31.25% |
| -30.00% portfolio shock | 70,000.00 | 25,000.00 | 45,000.00 | 35.71% |
| -50.00% portfolio shock | 50,000.00 | 25,000.00 | 25,000.00 | 50.00% |
| -70.00% portfolio shock | 30,000.00 | 25,000.00 | 5,000.00 | 83.33% |

This optional section is not about the company. It tests whether your own balance sheet can survive stress.

### Plain-English Meaning

Under the largest stress scenario shown here, the equity cushion would be 5,000.00 and loan/value would be 83.33%. This is about your own balance sheet, not the company.

---

## 12. Valuation Snapshot

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

### Plain-English Meaning

PE is around 37.4x, and price-to-sales is around 10.0x. The market is already pricing in a lot of future success, so the company needs strong execution to justify the valuation.

---

## 13. Research Score

[Open interactive score radar](AAPL_research_score_radar.html)

![AAPL Research Score Components](AAPL_research_score_components.png)

How to read this chart: the bars show which parts of the screening model help or hurt the score. They do not say whether the stock is cheap or safe.

| Component | Score | Weight | Profile |
| --- | --- | --- | --- |
| Growth Score | 40.75 / 100 | 14.00% | Mature Compounder |
| Profitability Score | 85.94 / 100 | 28.00% | Mature Compounder |
| Quality Trend Score | 50.75 / 100 | 18.00% | Mature Compounder |
| Risk Control Score | 65.43 / 100 | 18.00% | Mature Compounder |
| Benchmark Score | 58.22 / 100 | 14.00% | Mature Compounder |
| Valuation Sanity Score | 31.37 / 100 | 8.00% | Mature Compounder |
| Research Score | 61.34 / 100 | 100.00% | Mature Compounder |

### Beginner Warning

A high score is not a buy signal. A low score is not a sell signal. The score only helps prioritize further research under this model.

### Why This Score?

This score is a heuristic screening score weighted by research profile.
It is not a valuation model, not a prediction model, and not a buy/sell signal.

- Growth Score: revenue CAGR and latest revenue growth.
- Profitability Score: gross margin, operating margin, and FCF margin.
- Quality Trend Score: changes in gross margin, operating margin, and FCF margin.
- Risk Control Score: max drawdown, volatility, and beta.
- Benchmark Score: excess CAGR, information ratio, and Sharpe difference.
- Valuation Sanity Score: penalty-based check using PE, PS, EV/Revenue, and EV/EBITDA.

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


## AI Review

This section is AI-assisted and uses only the generated report data above.
It is not investment advice, not a buy/sell recommendation, and not a price target.

### Analyst Summary

The deterministic report presents AAPL as a mature, cash-generative business that outperformed SPY in raw return, while also showing weaker risk-adjusted efficiency and a demanding valuation setup.

### Evidence Check

- The Watchlist status is supported by strong profitability and cash generation, but the growth evidence is modest.
- The valuation concern is consistent with the reported PE and price-to-sales multiples.
- The benchmark comparison supports the idea that raw return was strong, but drawdown and Sharpe metrics make the risk picture less clean.

### Main Risks

- Future returns may depend heavily on maintaining premium valuation multiples.
- Revenue growth is not strong enough by itself to carry the thesis.
- Historical drawdown does not capture product-cycle, regulation, margin, or platform-risk shocks.

### Possible Beginner Misreadings

- A Watchlist score does not mean the stock should be bought.
- Outperforming SPY in the past does not prove future outperformance.
- Strong free cash flow does not automatically mean the stock is cheap.

### What to Verify Next

- Latest 10-K and 10-Q revenue mix, especially services contribution.
- Whether free cash flow is recurring or helped by working-capital timing.
- Margin durability, buyback pace, and valuation versus large-cap technology peers.

### Beginner Translation

AAPL looks like a strong business, but this report is not saying it is automatically attractive at any price. The main question is whether the current valuation already assumes too much future success.

### Confidence Note

This mocked sample demonstrates the AI Review format. A real AI Review should still be treated as a reasoning check, not as investment advice.

---

## 15. Manual Verification

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
- Sanity Checks HIGH severity items
- Ruin Risk debt and cash-burn assumptions

---

## 16. What to Check Next

Before making any serious judgment, manually check:

1. Latest 10-K / 10-Q revenue breakdown
2. Whether growth comes from volume, price, services, or accounting effects
3. Whether free cash flow is stable or one-off
4. Whether valuation is justified by future growth
5. Whether the company has hidden dilution, debt, or margin pressure

---

## 17. Final Research Questions

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

## 18. Generated Files

This folder contains CSV, chart, and Markdown outputs generated by the tool.

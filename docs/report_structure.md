# Report Structure / 报告结构

v2.0 reports are bilingual research radars. They are designed to surface opportunity cost, business quality, data reliability, and fragility risks without pretending to be buy/sell models.

v2.0 报告是中英双语研究雷达。目标是暴露机会成本、业务质量、数据可靠性和毁灭性风险，而不是假装自己能给出买卖结论。

## 0. Boundary / 边界

Defines what the report is not.

说明报告不提供买卖建议、目标价、收益承诺、交易信号或仓位建议。

## 1. One-line Verdict / 一句话判断

A concise human-readable summary of the ticker's current research posture.

一句话概括该标的目前更像什么：成熟复利股、投机成长股、数据不足案例、ETF 类标的，或需要警惕的高风险标的。

## 2. Key Takeaways / 核心摘要

Summarizes return, risk-adjusted performance, drawdown, growth quality, free cash flow, and valuation pressure.

总结收益、风险调整后表现、回撤、增长质量、自由现金流和估值压力。

## 3. Data Confidence / 数据可信度

Separates price data, company profile, financial statements, valuation fields, and segment revenue.

区分价格数据、公司资料、财务报表、估值字段和业务分部收入的可信度。

## 4. Sanity Scan / 主动断层扫描

The most important v2.0 reliability upgrade.

v2.0 最重要的可靠性升级。

Sanity Scan is not a soft disclaimer. It actively flags issues with severity, finding, and action:

Sanity Scan 不是一句“请自行核对”的免责声明，而是主动给出严重度、发现和动作建议：

- Currency mismatch / 币种不一致
- Short price history / 价格历史过短
- Short financial history / 财务历史过短
- Missing revenue / 营收缺失
- Missing FCF / 自由现金流缺失
- FCF consistency failure / 自由现金流一致性异常
- Fund-like instrument / 基金类标的
- Ruin-risk pressure / 毁灭性风险压力

## 5. Company Profile / 公司资料

Provides quick context from the data provider.

提供来自数据源的快速背景，但业务描述仍需核对年报、10-K、投资者材料和电话会纪要。

## 6. Price vs Benchmark / 价格与基准比较

Answers the opportunity-cost question:

回答一个核心问题：

```text
Why not simply hold the benchmark?
为什么不直接持有基准？
```

Charts include:

图表包括：

- Actual Close Price / 真实收盘价
- Normalized Performance / 归一化表现
- Drawdown / 回撤
- Interactive HTML dashboard / 交互式 HTML 图表

## 7. Ruin Risk Snapshot / 毁灭性风险快照

Separates ordinary price volatility from business fragility.

把普通价格波动和业务脆弱性分开。

Metrics include:

指标包括：

- Net Debt / 净债务
- EBITDA
- Net Debt / EBITDA
- Debt / FCF
- Cash Runway Years / 现金跑道年数
- Ruin Risk Score / 毁灭性风险分数

This section exists because historical max drawdown is not enough. A stock that once fell 30% can still later fall 100% if the business breaks.

这一节存在的原因是：历史最大回撤不够。一个曾经只跌过 30% 的股票，如果商业逻辑崩了，未来仍然可能归零。

## 8. Money Source and Money Flow / 钱从哪里来，流到哪里去

Shows revenue, gross profit, operating income, net income, operating cash flow, capex, and free cash flow.

展示营收、毛利、经营利润、净利润、经营现金流、资本开支和自由现金流。

This section is skipped for ETF / fund-like instruments.

ETF 或基金类标的会跳过该部分。

## 9. Valuation Snapshot / 估值快照

Grouped into:

分组展示：

- Market Size / 市值规模
- Valuation Multiples / 估值倍数
- Profitability and Growth / 盈利与增长
- Cash Flow and Debt / 现金流与债务
- Ownership / 股权结构
- Price Range / 价格区间

## 10. Personal Margin Stress / 个人融资压力测试

Optional account-level stress testing.

可选的个人账户压力测试。

If the user passes `--account-equity` and `--margin-loan`, the report estimates portfolio value, margin loan, equity cushion, and loan-to-value under several drawdown scenarios.

如果用户传入 `--account-equity` 和 `--margin-loan`，报告会估算不同下跌情景下的组合价值、融资余额、权益缓冲和融资比例。

This section is intentionally personal. A stock can be analytically interesting and still be dangerous for a leveraged account.

这一节是刻意个人化的：一只股票可以很值得研究，但仍然可能不适合一个有融资压力的账户。

## 11. Research Potential Score / 研究潜力评分

v2.0 uses research profiles instead of one universal score.

v2.0 不再假装所有公司都适合一个评分模板，而是使用研究类型加权。

Profiles include:

研究类型包括：

- Mature Compounder / 成熟复利型
- Profitable Growth / 盈利成长型
- Speculative Growth / 投机成长型
- Cyclical / Asset Heavy / 周期或重资产型
- Financials / 金融类
- ETF / Fund / 基金类
- Data Limited / 数据不足

The score remains a heuristic. It is not a valuation model, prediction model, or investment recommendation.

评分仍然只是启发式研究优先级，不是估值模型、预测模型或投资建议。

## 12. Required Manual Verification / 必须人工核对

The report explicitly lists what must be checked manually:

报告明确列出必须人工核对的内容：

- Revenue source and segment breakdown / 收入来源和分部拆解
- Gross margin trend / 毛利率趋势
- Free cash flow calculation / 自由现金流计算
- Debt and dilution / 债务和稀释
- Stock-based compensation / 股权激励
- One-time gains or losses / 一次性损益
- Guidance / 管理层指引
- SEC 10-K / 10-Q
- Company IR materials / 公司投资者材料

## 13. Final Research Questions / 最后必须回答

The final section forces judgment back to the human.

最后一节把判断权交还给人。

Examples:

- Why not simply buy the benchmark?
- Has the stock earned its extra risk?
- If the stock falls 70%, does the business survive without destructive dilution?
- Is the company being judged against the right lifecycle and sector peers?

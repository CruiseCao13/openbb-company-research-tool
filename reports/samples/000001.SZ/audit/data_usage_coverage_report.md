# Data Usage Coverage Report

This audit checks whether fetched critical data has a clear destination. The goal is not to chart everything; it is to avoid leaving important evidence unexplained.

## Field Destination Matrix

| Fetched data field | Fetched | Used in report | Used in chart | Used in table | Used in appendix | Unused reason |
|---|---:|---:|---:|---:|---:|---|
| price history | false | false | false | false | true | Price history missing; price/drawdown charts become data gap cards. |
| revenue | false | false | false | false | true | Revenue missing; financial trend table/chart cannot prove growth. |
| operating income | false | false | false | false | true | Operating income missing; margin quality remains less verifiable. |
| net income | false | false | false | false | true | Net income missing; profitability checks are limited. |
| operating cash flow | false | false | false | false | true | Operating cash flow missing; money-flow analysis is degraded. |
| capex | false | false | false | false | true | Capex missing; free-cash-flow bridge cannot be fully verified. |
| free cash flow | false | false | false | false | true | Free cash flow missing; report explains operating cash flow and capex separately. |
| cash | false | false | false | false | true | Cash missing; runway and liquidity checks are limited. |
| debt | false | false | false | false | true | Debt missing; leverage checks are limited. |
| shares | false | false | false | false | true | Share count missing; dilution/buyback quality needs manual verification. |
| valuation multiples | false | false | false | false | true | Valuation multiples missing or not meaningful for this profile. |

## Critical Unused Fields

- None. Fetched critical fields have a destination in report, chart, table, or appendix.

## Missing Critical Fields

- price history
- revenue
- operating income
- net income
- operating cash flow
- capex
- free cash flow
- cash
- debt
- shares
- valuation multiples
- Provider payload has an error; numeric conclusions must stay screening-only.

## Industry Critical Fields

- 营业收入
- 归母净利润
- 扣非净利润
- 经营现金流
- 货币资金
- 有息负债
- ROE

## Validator Impact

PASS

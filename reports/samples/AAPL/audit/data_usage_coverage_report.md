# Data Usage Coverage Report

This audit checks whether fetched critical data has a clear destination. The goal is not to chart everything; it is to avoid leaving important evidence unexplained.

## Field Destination Matrix

| Fetched data field | Fetched | Used in report | Used in chart | Used in table | Used in appendix | Unused reason |
|---|---:|---:|---:|---:|---:|---|
| price history | true | true | true | false | true |  |
| revenue | true | true | true | true | true |  |
| operating income | true | true | true | true | true |  |
| net income | true | true | false | true | true |  |
| operating cash flow | true | true | true | true | true |  |
| capex | true | true | true | true | true |  |
| free cash flow | true | true | true | true | true |  |
| cash | false | false | false | false | true | Cash missing; runway and liquidity checks are limited. |
| debt | true | true | false | true | true |  |
| shares | true | true | false | false | true |  |
| valuation multiples | true | true | true | true | true |  |
 
## Critical Unused Fields

- None. Fetched critical fields have a destination in report, chart, table, or appendix.

## Missing Critical Fields

- cash

## Industry Critical Fields

- No extra industry-critical field set triggered.

## Validator Impact

PASS

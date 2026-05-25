# Regression Hard Cases Check

- Batch root: `reports/batch_runs/regression_hard_cases_local_check`
- Exists: True
- Rows: 16

## Case matrix
| Ticker | Status | Frame | Failed checks |
|---|---|---|---|
| LUNR | WARNING | Speculative Aerospace / Space Systems |  |
| ASTS | WARNING | Speculative Aerospace / Space Systems |  |
| AAPL | WARNING | Mature Consumer Technology Compounder |  |
| GOOGL | WARNING | Platform Internet / Digital Ads / Cloud |  |
| META | WARNING | Platform Internet / Social Ads / AI Infrastructure |  |
| CAT | WARNING | Cyclical / Industrial Cycle |  |
| ISRG | WARNING | Biotech / Pharma Research Frame |  |
| LLY | WARNING | Biotech / Pharma Research Frame |  |
| AMD | WARNING | AI Semiconductor / Data Center Growth Compounder |  |
| NVDA | WARNING | AI Semiconductor / Data Center Growth Compounder |  |
| T | WARNING | Telecom / Infrastructure Cash Flow |  |
| ZIM | WARNING | Shipping / Airlines / Transport Cycle |  |
| INTC | WARNING | Capital-Intensive Semiconductor Turnaround |  |
| JPM | WARNING | Financials / Bank-like Screening |  |
| 600519.SH | WARNING | Consumer / Retail |  |
| 000001.SZ | WARNING | Financials / Bank-like Screening |  |

## LUNR forbidden terms
- LUNR frame: Speculative Aerospace / Space Systems
- Telecom / Infrastructure Cash Flow: False
- wireless service revenue: False
- broadband / network revenue: False
- subscriber churn: False
- LUNR wrong framework local check: PASS

Expected ticker rows: 16

Final status: PASS

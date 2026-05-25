# A-share Sector Smoke Report

Date: 2026-05-25

## Scope

This smoke pass used the v5 Rust CLI with the A-share `auto` provider path and local AI mode only. It did not call external OpenAI API.

Command pattern:

```bash
cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- run TICKER \
  --market cn \
  --provider auto \
  --ai local \
  --run-id a_share_sector_smoke_TICKER \
  --force
```

All runs are local fallback analysis and must not be treated as external OpenAI analysis.

## Provider Source Summary

All eight tickers returned real public-provider payloads with the same source labeling:

- Provider: `eastmoney_public`
- Source: `Eastmoney public endpoint`
- Provider adapter: `akshare_compatible_fallback`
- Package used: `false`
- Mock: `false`
- Market: `CN_A`
- Currency: `CNY`

The installed optional provider packages were not used in these smoke runs. The fallback is real public data, not mock data, but it is not a guaranteed official data contract.

## Sector Smoke Results

| Ticker | Company | Expected Frame | Actual Frame | Report Status | Provider Data | Missing Fields | Result |
|---|---|---|---|---|---|---|---|
| 600519.SH | 贵州茅台酒股份有限公司 | Consumer / premium liquor | A-share Premium Baijiu / Consumer Brand | PASS | profile, price, income, balance, cash flow present | 货币资金余额, 分红 | PASS |
| 000001.SZ | 平安银行股份有限公司 | Bank-like financials | Financials / Bank-like Screening | PASS | profile, price, income, balance, cash flow present | none | PASS |
| 300750.SZ | 宁德时代新能源科技股份有限公司 | New energy / battery manufacturing | Unknown / Data-Limited Screening | WARNING | profile, price, income, balance, cash flow present | 货币资金余额, 分红 | WARNING |
| 600036.SH | 招商银行股份有限公司 | Bank-like financials | Financials / Bank-like Screening | PASS | profile, price, income, balance, cash flow present | none | PASS |
| 601318.SH | 中国平安保险(集团)股份有限公司 | Insurance | Unknown / Data-Limited Screening | WARNING | profile, price, income, balance, cash flow present | 毛利率, 存货, 应收账款, 货币资金余额, 分红 | WARNING |
| 000858.SZ | 宜宾五粮液股份有限公司 | Consumer / premium liquor | A-share Premium Baijiu / Consumer Brand | PASS | profile, price, income, balance, cash flow present | 货币资金余额, 分红 | PASS |
| 600276.SH | 江苏恒瑞医药股份有限公司 | Pharma | Unknown / Data-Limited Screening | WARNING | profile, price, income, balance, cash flow present | 货币资金余额, 分红 | WARNING |
| 601899.SH | 紫金矿业集团股份有限公司 | Materials / mining | Unknown / Data-Limited Screening | WARNING | profile, price, income, balance, cash flow present | 货币资金余额, 分红 | WARNING |

## Data Coverage

Each provider payload contained:

- company profile
- price history
- income statement
- balance sheet
- cash flow
- `market=CN_A`
- `currency=CNY`
- provider status and missing fields

Each run wrote:

- `raw/provider_payload.json`
- `metadata/provider_status.json`
- `metadata/company_understanding.json`
- `metadata/report_status.json`
- `metadata/ai_usage.json`
- `report/*.md`

## Run Paths

- `reports/600519.SH/runs/a_share_sector_smoke_600519_SH`
- `reports/000001.SZ/runs/a_share_sector_smoke_000001_SZ`
- `reports/300750.SZ/runs/a_share_sector_smoke_300750_SZ`
- `reports/600036.SH/runs/a_share_sector_smoke_600036_SH`
- `reports/601318.SH/runs/a_share_sector_smoke_601318_SH`
- `reports/000858.SZ/runs/a_share_sector_smoke_000858_SZ`
- `reports/600276.SH/runs/a_share_sector_smoke_600276_SH`
- `reports/601899.SH/runs/a_share_sector_smoke_601899_SH`

## Findings

PASS:

- Baijiu companies were correctly framed as premium liquor / consumer brands.
- Banks were correctly framed as bank-like financials.
- Provider source labeling was clear and did not confuse Eastmoney public fallback with `akshare_package`.
- No mock provider data was used.

WARNING:

- CATL (`300750.SZ`) provider profile clearly contains lithium battery and energy-storage language, but local AI downgraded it to `Unknown / Data-Limited Screening` instead of a new energy / battery manufacturing frame.
- Ping An (`601318.SH`) provider profile clearly says insurance, but local AI downgraded it to `Unknown / Data-Limited Screening` instead of an insurance frame.
- Hengrui Pharma (`600276.SH`) provider profile clearly says pharmaceutical manufacturing, but local AI downgraded it to `Unknown / Data-Limited Screening`.
- Zijin Mining (`601899.SH`) provider profile clearly says mining and nonferrous metals, but local AI downgraded it to `Unknown / Data-Limited Screening`.

These are not provider-data failures. They are local sector-frame coverage gaps in the fallback analyst. The system did not falsely mark them as clean PASS; the affected reports have `WARNING`, `LOW` confidence, and `human_review_required=true`.

## Conclusion

A-share sector smoke status: WARNING.

The A-share provider path returned usable real public data for all eight tickers with clear source labeling, but sector-frame coverage is not complete. Do not claim full A-share support from this smoke pass. The next fix should add deterministic or prompt-level A-share sector guards for battery manufacturing, insurance, pharma, and mining before treating these sectors as product-ready.

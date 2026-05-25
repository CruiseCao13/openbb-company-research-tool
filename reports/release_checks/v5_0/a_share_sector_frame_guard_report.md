# A-share Sector Frame Guard Report

Date: 2026-05-25

## Scope

This pass added deterministic A-share sector frame guards for local/company understanding. It did not change provider source logic, dashboard/PDF rendering, or broad_500 behavior.

Provider mode used for verification:

- `--provider auto`
- `--ai local`
- No external OpenAI API calls

## Guard Coverage Added

The local compact analyst now uses ticker, company name, sector, industry, and provider business description to identify:

- `300750.SZ` / CATL-like battery and new energy manufacturing companies
- `601318.SH` / Ping An-like insurance and integrated financial companies
- `600276.SH` / Hengrui-like pharma and innovative drug portfolio companies
- `601899.SH` / Zijin-like mining, nonferrous metals, and commodity-cycle companies

The guards also add explicit `must_not` boundaries so these companies are not treated as banks, insurers, consumer brands, software platforms, biotech-only cash-runway stories, or industrial FCF stories when that frame is wrong.

## Verification Results

| Ticker | Provider | Source | Package Used | Mock | Currency | Final Frame | Status | Missing Fields |
|---|---|---|---:|---:|---|---|---|---|
| 600519.SH | eastmoney_public | Eastmoney public endpoint | false | false | CNY | A-share Premium Baijiu / Consumer Brand | PASS | 货币资金余额, 分红 |
| 000001.SZ | eastmoney_public | Eastmoney public endpoint | false | false | CNY | Financials / Bank-like Screening | PASS | none |
| 300750.SZ | eastmoney_public | Eastmoney public endpoint | false | false | CNY | New Energy / Battery Manufacturing | PASS | 货币资金余额, 分红 |
| 600036.SH | eastmoney_public | Eastmoney public endpoint | false | false | CNY | Financials / Bank-like Screening | PASS | none |
| 601318.SH | eastmoney_public | Eastmoney public endpoint | false | false | CNY | Insurance / Integrated Financials | PASS | 毛利率, 存货, 应收账款, 货币资金余额, 分红 |
| 000858.SZ | eastmoney_public | Eastmoney public endpoint | false | false | CNY | A-share Premium Baijiu / Consumer Brand | PASS | 货币资金余额, 分红 |
| 600276.SH | eastmoney_public | Eastmoney public endpoint | false | false | CNY | Pharma / Innovative Drug Portfolio | PASS | 货币资金余额, 分红 |
| 601899.SH | eastmoney_public | Eastmoney public endpoint | false | false | CNY | Mining / Nonferrous Metals / Commodity Cycle | PASS | 货币资金余额, 分红 |

## Run Paths

- `reports/600519.SH/runs/a_share_sector_frame_guard_600519_SH`
- `reports/000001.SZ/runs/a_share_sector_frame_guard_000001_SZ`
- `reports/300750.SZ/runs/a_share_sector_frame_guard_300750_SZ`
- `reports/600036.SH/runs/a_share_sector_frame_guard_600036_SH`
- `reports/601318.SH/runs/a_share_sector_frame_guard_601318_SH`
- `reports/000858.SZ/runs/a_share_sector_frame_guard_000858_SZ`
- `reports/600276.SH/runs/a_share_sector_frame_guard_600276_SH`
- `reports/601899.SH/runs/a_share_sector_frame_guard_601899_SH`

## Tests Added

- `cn_catl_battery_not_unknown`
- `cn_catl_not_bank_or_consumer`
- `cn_pingan_insurance_not_unknown`
- `cn_pingan_not_industrial_fcf`
- `cn_hengrui_pharma_not_unknown`
- `cn_hengrui_not_early_biotech_only`
- `cn_zijin_mining_not_unknown`
- `cn_zijin_not_biotech_or_software`
- `a_share_sector_frame_guard_uses_cn_ticker_and_profile`
- `a_share_unknown_only_when_profile_missing_or_conflicting`

## Remaining Limitations

- These were local fallback runs, not external OpenAI analyses.
- Provider source remains `eastmoney_public`; important values still require manual verification against filings or company reports.
- Some optional fields remain unavailable from the public fallback, especially cash balance and dividend detail for non-bank companies.
- Insurance-specific public-provider fields remain incomplete; the frame is now correct, but deeper insurance analysis still needs premium, underwriting, solvency, embedded value, and investment portfolio details.

## Result

A-share sector frame guard status: PASS for the eight-ticker smoke set.

This pass fixes the prior WARNING where provider data was present but local/company frame degraded to `Unknown / Data-Limited Screening` for CATL, Ping An, Hengrui, and Zijin.

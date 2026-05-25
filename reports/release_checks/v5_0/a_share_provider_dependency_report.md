# A-share Provider Dependency and Source Labeling Report

Date: 2026-05-25

## Dependency Install Status

| Package | Installed | Version |
|---|---:|---|
| akshare | yes | 1.18.63 |
| tushare | yes | 1.4.29 |
| baostock | yes | 00.9.10 |

`TUSHARE_TOKEN` set: no

## Provider Health Summary

- AKShare package installed: yes
- Tushare package installed: yes
- Baostock package installed: yes
- Eastmoney public fallback available: yes
- Missing `TUSHARE_TOKEN` is a warning, not a default CI failure.
- Mock data used: false

The current A-share auto path uses `eastmoney_public` with `provider_adapter=akshare_compatible_fallback`. It does not label Eastmoney public data as `akshare_package`.

## 600519.SH Verification

Run path: `reports/600519.SH/runs/a_share_provider_package_check_600519`

| Field | Value |
|---|---|
| Provider used | eastmoney_public |
| Source | Eastmoney public endpoint |
| Provider adapter | akshare_compatible_fallback |
| Package used | false |
| Mock | false |
| Market | CN_A |
| Currency | CNY |
| Provider status | PASS |
| Price rows | 260 |
| Income rows | 64 |
| Balance rows | 152 |
| Cash flow rows | 32 |
| Missing fields | 货币资金余额, 分红 |

Important values should still be checked against exchange filings or company reports because the fallback uses public endpoints with schema and coverage limitations.

## 000001.SZ Verification

Run path: `reports/000001.SZ/runs/a_share_provider_package_check_000001`

| Field | Value |
|---|---|
| Provider used | eastmoney_public |
| Source | Eastmoney public endpoint |
| Provider adapter | akshare_compatible_fallback |
| Package used | false |
| Mock | false |
| Market | CN_A |
| Currency | CNY |
| Provider status | PASS |
| Price rows | 260 |
| Income rows | 64 |
| Balance rows | 152 |
| Cash flow rows | 32 |
| Missing fields | none |

## Disclosure Checks

- `raw/provider_payload.json` includes `metadata.source`, `provider_version`, `package_used`, `mock`, `provider_adapter`, `provider_limitations`, and `data_quality_warnings`.
- `metadata/provider_status.json` includes provider source, adapter, package usage, mock status, currency, market, missing fields, and limitations.
- Markdown reports display provider source, adapter, package usage, mock status, missing fields, and limitations.
- Dashboards display an A-share provider source card with the same fields.
- Pack outputs do not include `.env`, API keys, or provider cache files.

## Remaining Limitations

- The installed AKShare package is available, but the default auto path did not use package endpoints in this verification because the public fallback is faster and avoids package endpoint hangs in this environment.
- The price history came from a Sina public price endpoint after Eastmoney price endpoint instability; the provider remains labeled `eastmoney_public` with public-endpoint limitations disclosed.
- Tushare package is installed, but Tushare is not used because `TUSHARE_TOKEN` is not set.
- Baostock package is installed, but it was not needed for these two successful fallback runs.

## Result

A-share minimum provider blocker resolved: yes, with explicit public-fallback labeling and remaining data limitations disclosed.

# Provider and Data Coverage Audit

Generated: 2026-05-25 Asia/Singapore

## Coverage Matrix

| Run | Provider | Market | Provider Status | Company Name | Price Rows | Income Rows | Balance Rows | Cash Flow Rows | Valuation |
|---|---|---|---|---|---:|---:|---:|---:|---|
| reports/AAPL/runs/path_audit_aapl | yfinance | US | PASS | Apple Inc. | 260 | 195 | 200 | 200 | True |
| reports/600519.SH/runs/final_audit_600519 | akshare | CN_A | PASS | 600519.SH | 0 | 0 | 0 | 0 | False |
| reports/000001.SZ/runs/final_audit_000001 | akshare | CN_A | PASS | 000001.SZ | 0 | 0 | 0 | 0 | False |

## Required Artifacts

All audited runs have `raw/provider_payload.json`, `metadata/provider_status.json`, `audit/provider_validation.md`, `audit/data_inventory_report.md`, `audit/data_usage_coverage_report.md`, and `metadata/unit_policy.json`.

## Finding

AAPL has usable yfinance/OpenBB-style locked data. The audited A-share AKShare runs generated data-limited reports and did not pretend to verify financial statements, but their `metadata/provider_status.json` still says `PASS` even when price and financial arrays are empty. The rendered report downgrades to `Provider status | PROVIDER_ERROR`, so the user-facing report is honest, but provider metadata should be aligned in the next fix.

Status: FAIL

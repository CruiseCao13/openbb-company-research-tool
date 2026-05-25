# v5 Report Structure Consistency Report

Generated: 2026-05-25 Asia/Singapore

## Scope

Searched v5 sample reports for old v4-style report markers:

```bash
rg -n "Beginner Summary|Research Score|v4|asset profile|AI Review Gate|Language Lint Gate|Mature Compounder|How to Read This Report" reports/samples reports/AAPL/runs/api_verify_aapl_real
```

## Findings

| Check | Result | Notes |
|---|---|---|
| Old v4 markers in v5 samples | PASS | No matches found in `reports/samples` or the real AAPL API verification report path. |
| v5 status block | PASS | Sample reports display report status and AI source fields. |
| Company identity section | PASS | Sample reports include `## 2. Company Identity`. |
| Business model section | PASS | Sample reports include `## 3. Business Model`. |
| Money flow section | PASS | Sample reports include `## 4. Money Flow...`. |
| Financial interpretation section | PASS | Sample reports include `## 5. Financial Statement Interpretation`. |
| Research blueprint section | PASS | Sample reports include `## 6. AI Research Blueprint`. |
| Data gaps and self-review | PASS | Sample reports include data gaps and AI self-review sections. |
| Locked data appendix | PASS | Sample reports include an appendix with locked data. |

## Note

The current v5 renderer includes a dedicated `Charts and Evidence` section before the locked-data appendix. This is v5 evidence presentation, not a legacy v4 report structure.

## Result

v5 samples do not use the old v4/v3/v2 report structure.

Status: PASS

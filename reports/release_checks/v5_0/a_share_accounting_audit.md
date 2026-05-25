# A-share Accounting and Unit Audit

Generated: 2026-05-25 Asia/Singapore

## Scan Evidence

```text
reports/samples/600519.SH/report/600519.SH_research_report_cn.md:168:What to look at：看经营现金流、资本开支和融资压力。
```

## Findings

| Check | Status | Notes |
|---|---|---|
| A-share reports avoid 10-K/10-Q/SEC core-source language | PASS | No 10-K/10-Q/SEC matches in audited A-share reports. |
| A-share reports avoid USD/RMB mixing without explanation | PASS | No conflicting currency claims found in audited reports. |
| A-share provider data available | FAIL | AKShare audit runs returned empty price and financial arrays. |
| A-share accounting fields represented | WARNING | Because provider data is empty, reports remain `Unknown / Data-Limited Screening`; they do not yet demonstrate full A-share accounting coverage. |
| Report honesty | PASS | Reports mark WARNING / provider error / data-limited instead of full verified analysis. |

Status: FAIL

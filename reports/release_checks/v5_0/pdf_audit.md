# PDF Audit

Generated: 2026-05-25 Asia/Singapore

| Run/Sample | PDF Count | PDF File | pdf_export_report.md | pdf_status.json |
|---|---:|---|---|---|
| reports/samples/AAPL | 2 | PASS | PASS | PASS |
| reports/samples/GOOGL | 1 | PASS | WARNING | WARNING |
| reports/samples/CAT | 1 | PASS | WARNING | WARNING |
| reports/samples/AMD | 1 | PASS | WARNING | WARNING |
| reports/samples/600519.SH | 2 | PASS | WARNING | WARNING |
| reports/samples/000001.SZ | 1 | PASS | PASS | PASS |
| reports/AAPL/runs/api_verify_aapl_real | 1 | PASS | PASS | PASS |

Some samples have PDF files but do not carry `audit/pdf_export_report.md`; the external AAPL verification run and AAPL/000001 samples do include PDF audit reports. This is a product-polish warning, not a runtime blocker for report generation.

Status: WARNING

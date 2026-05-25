# Security / Privacy Audit

Generated: 2026-05-25 Asia/Singapore

## Secret Scan

See `secret_safety_report.md`; no full API key was found. `.env` and `.env.*` are ignored except `.env.example`.

## Privacy Findings

| Check | Status | Notes |
|---|---|---|
| API key in reports/dashboard/PDF | PASS | No `sk-...` key pattern found. |
| `.env` committed | PASS | `.env` ignored and not tracked. |
| AI usage files expose key | PASS | `ai_usage.json` records model/source only. |
| Local absolute paths in audit reports | WARNING | Release-check evidence includes local filesystem paths for reproducibility in this workspace. |
| Pack includes cache | PASS | Pack manifest excludes cache directories. |

Status: WARNING

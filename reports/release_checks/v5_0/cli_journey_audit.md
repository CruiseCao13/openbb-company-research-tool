# CLI Journey Audit

Generated: 2026-05-25 Asia/Singapore

| Journey | Command / Evidence | Status |
|---|---|---|
| Doctor | `cargo run -p research-rs -- doctor` wrote `reports/release_checks/provider_health.md` | PASS |
| AAPL local run | `reports/AAPL/runs/path_audit_aapl` | PASS |
| AAPL external run | Existing verified run `reports/AAPL/runs/api_verify_aapl_real` with 4 external OpenAI calls | PASS |
| 600519.SH A-share run | `reports/600519.SH/runs/final_audit_600519` | WARNING: data-limited provider output |
| broad_30 batch | `reports/batch_runs/v5_consistency_broad_30_audit` | PASS |
| pack report | `reports/AAPL/runs/path_audit_aapl/pack/AAPL_research_pack.zip` | PASS |
| dashboard path check | run dashboards exist for audited runs | PASS |

Status: WARNING

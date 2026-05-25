# README / Docs Consistency Audit

Generated: 2026-05-25 Asia/Singapore

## Scan Evidence

```text
README.md:276:Earlier v2-v4 Python workflows remain available for compatibility; see [docs/history_v2_v4.md](docs/history_v2_v4.md). Legacy commands such as `openbb-research`, `cresearch`, and `python scripts/company_research_tool.py` are not the v5 primary entry point.
docs/output_folder_structure.md:3:v4.3 run folders are organized as report packs.
docs/cli_usage.md:10:openbb-research AAPL
docs/cli_usage.md:11:cresearch AAPL
docs/cli_usage.md:18:openbb-research RKLB --both --full
docs/cli_usage.md:19:openbb-research AAPL --en
docs/cli_usage.md:20:openbb-research AAPL --zh
docs/cli_usage.md:21:openbb-research NVDA --benchmark QQQ
docs/cli_usage.md:22:openbb-research RKLB --run-id stress_rklb_v43
docs/cli_usage.md:23:openbb-research INTC --both --full --pack --run-id stress_intc_v43
docs/cli_usage.md:24:openbb-research pack reports/INTC/runs/stress_intc_v43
docs/cli_usage.md:29:The v4.3 Python workflow used these defaults:
docs/cli_usage.md:41:openbb-research pack reports/TICKER/runs/RUN_ID
docs/v4_3_report_system_design.md:1:# v4.3 Report System Design
docs/v4_3_report_system_design.md:3:v4.3 changes the project from a polished report renderer into an asset-aware research workflow.
docs/testing_matrix.md:3:v4.3 tests focus on report logic rather than only string formatting.
docs/batch_evaluation.md:7:v4.4 adds a batch evaluation foundation for cross-industry regression and local system-training cases.
docs/batch_evaluation.md:16:openbb-research batch eval_sets/smoke_12.yaml --both --full --pack --max-workers 2 --no-ai
docs/batch_evaluation.md:17:openbb-research batch eval_sets/smoke_12.yaml --ai-review-failures --max-ai-reviews 5 --resume
docs/batch_evaluation.md:23:openbb-research batch eval_sets/broad_200.yaml --both --full --pack --max-workers 2 --no-ai
docs/batch_evaluation.md:29:openbb-research batch eval_sets/broad_200.yaml --ai-review-failures --max-ai-reviews 40 --resume
docs/known_issues_and_roadmap.md:3:## v4.3 Status Note
docs/known_issues_and_roadmap.md:5:v4.3 adds asset-aware routing, thesis spine generation, profile-specific interpretation blocks, AI interpretation patch artifacts, organized report packs, and automatic self-review files.
docs/known_issues_and_roadmap.md:530:Known presentation issues found after v3.0:
docs/known_issues_and_roadmap.md:641:### v3.0.1 — Documentation and Report Polish
docs/known_issues_and_roadmap.md:651:- preserve v3.0 architecture
docs/known_issues_and_roadmap.md:732:1. Keep v3.0 stable.
docs/development_log.md:3:## v4.3 — Asset-Aware Research Report System
docs/development_log.md:13:v4.3 introduced:
docs/development_log.md:34:The fix was not ticker-specific. v4.3 now includes a generalized `Capital-Intensive Semiconductor Turnaround` route based on sector, industry, business-summary clues, capex intensity, margin pressure, weak or negative FCF, and unstable profitability.
docs/development_log.md:59:## 1. v2.0 — From Data Pack to Research Workflow
docs/development_log.md:80:v2.0 introduced:
docs/development_log.md:101:The v2.0 report became more powerful, but it still risked feeling like a professional metric dump.
docs/development_log.md:132:## 3. v3.0 — Optional AI Review Layer
docs/development_log.md:151:v3.0 added:
docs/development_log.md:301:.venv/bin/python scripts/company_research_tool.py AAPL --benchmark SPY --start 2023-01-01 --ai-review --no-rich
docs/development_log.md:369:## 8. Report Quality Issues Found After v3.0
docs/development_log.md:373:The v3.0 report is usable and presentable, but several quality issues remain:
docs/v2_critical_review.md:69:Current setup writes a `cresearch` wrapper into `~/.local/bin`.
docs/history_v2_v4.md:10:openbb-research AAPL --both --full
docs/history_v2_v4.md:11:cresearch AAPL --both --full
docs/history_v2_v4.md:12:python scripts/company_research_tool.py AAPL
docs/history_v2_v4.md:22:- v4.4 introduced the batch evaluation foundation and system-training case generation.
docs/report_structure.md:7:## v4.3 Asset-Aware Report Pack
docs/report_structure.md:9:v4.3 reports begin with an asset profile and thesis spine before rendering the main memo. This prevents a speculative-growth company from receiving mature-compounder wording, and it prevents unknown companies from being forced into a confident template.
docs/report_structure.md:11:The v4.3 report pack contains:
docs/report_structure.md:40:Older notes below describe the v4.0/v4.2 reading model and remain useful background.
docs/performance_notes.md:3:v4.3 keeps the core workflow in Python.
docs/performance_notes.md:13:No Rust or C++ sidecar was added in v4.3.
```

## Classification

- README is v5-first, bilingual, uses `research-rs` in Quick Start, includes US + CN samples, external AI proof, limitations, and disclaimer.
- Old Python commands appear in README only in the Legacy Python Workflow paragraph.
- `docs/history_v2_v4.md`, `docs/cli_usage.md`, `docs/batch_evaluation.md`, and `docs/report_structure.md` are marked as historical or legacy.
- Development logs and old design docs retain historical references by design.

Status: PASS

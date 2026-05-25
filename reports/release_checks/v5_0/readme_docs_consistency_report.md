# README and Docs Consistency Report

Generated: 2026-05-25

## README Current Product Check

Strict grep:

```bash
grep -nE "v4\.3|v4\.4|v4\.2|v3\.0|v2\.0|openbb-research|cresearch|company_research_tool.py|asset-aware workflow|v4 Workflow Gates" README.md
```

Result: PASS, no matches.

## Required Content

| Requirement | Status |
| --- | --- |
| v5 title | PASS |
| Rust-powered AI-led positioning | PASS |
| Bilingual English/Chinese content | PASS |
| `research-rs` primary entry | PASS |
| external AI usage proof through `ai_usage.json` | PASS |
| local/mock/external distinction | PASS |
| US + CN sample gallery | PASS |
| limitations and not-investment-advice disclaimer | PASS |
| roadmap | PASS |
| legacy link only | PASS |

Final status: PASS

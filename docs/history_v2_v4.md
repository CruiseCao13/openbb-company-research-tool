# History: v2-v4 Python Workflows

The current primary product is the v5 Rust-powered AI-led research engine. This file preserves the earlier Python workflow history so the main README can stay focused on v5.

## Legacy Entry Points

The older Python workflow used commands such as:

```bash
openbb-research AAPL --both --full
cresearch AAPL --both --full
python scripts/company_research_tool.py AAPL
```

These commands may remain available for compatibility, regression comparison, or legacy report reproduction. They are not the primary v5 entry point.

## v2-v4 Direction

- v2 focused on producing structured company research outputs from provider data.
- v3 improved report packaging, AI-skipped/fallback handling, and basic dashboard-style outputs.
- v4 improved asset-aware routing, profile-specific report language, deterministic linting, and batch evaluation.
- v4.4 introduced the batch evaluation foundation and system-training case generation.

## Why v5 Replaced the Main Narrative

The v2-v4 Python workflow still wrote reports from a rule/template-first posture. v5 changes the control flow:

```text
Data -> Locked numbers -> AI company understanding -> AI research blueprint -> Validator -> Report -> AI self-review -> Batch eval
```

Use the v5 README for the current product surface and `research-rs` as the primary CLI.

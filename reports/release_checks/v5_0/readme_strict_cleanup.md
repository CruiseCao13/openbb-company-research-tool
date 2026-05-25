# README Strict Cleanup Review

Generated: 2026-05-25

## Scope

This review covers the strict README cleanup requested for v5.0 product positioning.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| Old v4 body removed from README | PASS | README no longer contains old current-product sections such as v4 asset-aware workflow, old demo, old Python quick start, or old report structure. |
| Legacy content moved to history doc | PASS | Historical Python workflow material is referenced through `docs/history_v2_v4.md`. |
| README primary path is research-rs | PASS | Quick Start uses `cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- ...`. |
| README has US + CN samples | PASS | Sample gallery includes AAPL, RKLB, GOOGL, CAT, AMD, 600519.SH, and 000001.SZ. |
| README explains external AI proof | PASS | README points users to `metadata/ai_usage.json` and names `external_ai_used`, `local_mock_used`, `new_external_ai_calls`, `cache_hits`, `model`, and `tasks`. |
| README is bilingual | PASS | README has English product overview and Chinese product explanation, plus Chinese and English disclaimers. |
| Legacy commands are not primary | PASS | README does not include legacy Python command names; it links to `docs/history_v2_v4.md` instead. |

## Validation

Strict grep target for README:

```text
v4.3|v4.4|v4.2|v3.0|v2.0|openbb-research|cresearch|company_research_tool.py|asset-aware workflow|v4 Workflow Gates
```

Expected result: no matches in `README.md`.

## README UX

README UX: PASS

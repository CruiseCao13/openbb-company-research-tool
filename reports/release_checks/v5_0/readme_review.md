# README Review

## Scope

Reviewed `README.md` after the final v5.0 product-homepage rewrite.

## Checklist

| Check | Status | Evidence |
|---|---|---|
| v5 is clearly primary | PASS | README title is `OpenBB Company Research Tool v5.0`; primary entry point is `research-rs`. |
| No old v4 current narrative remains | PASS | Removed old main README sections such as 30-Second Demo, v4 workflow gates, v4.3/v4.4 product sections, and v2/v3/v4 improvement blocks. |
| Legacy Python workflow is clearly marked legacy | PASS | Legacy commands appear only under `Legacy Python Workflow` and point to `docs/history_v2_v4.md`. |
| Bilingual content is present | PASS | README has separate `English Product Overview` and `中文产品说明` sections. |
| v5 architecture is explained | PASS | README includes Mermaid pipeline plus Rust/Python/AI/Validator responsibility split. |
| Primary quick start uses research-rs | PASS | Quick Start uses `source "$HOME/.cargo/env"`, `cd research-rs`, and `cargo run -p research-rs`. |
| External AI proof is explained | PASS | README explains `metadata/ai_usage.json` and fields `external_ai_used`, `local_mock_used`, `new_external_ai_calls`, `cache_hits`, `model`, and `tasks`. |
| US + CN samples exist | PASS | README links AAPL, GOOGL, CAT, AMD, 600519.SH, and 000001.SZ samples. |
| Sample links exist | PASS | Required report, dashboard, `ai_usage`, company understanding, blueprint, and self-review links exist. |
| Dashboard/PDF/charts are described | PASS | README includes Dashboard, PDF, and Charts section. |
| Content quality evaluation is described | PASS | README includes product quality, evidence map, data inventory, visual lint, chart/table report, and training cases. |
| Limitations and disclaimer are honest | PASS | README states no investment advice, no target price, provider limitations, AI fallibility, and local fallback boundaries. |

## Sample Link Evidence

Checked:

- `reports/samples/AAPL/`
- `reports/samples/GOOGL/`
- `reports/samples/CAT/`
- `reports/samples/AMD/`
- `reports/samples/600519.SH/`
- `reports/samples/000001.SZ/`

Each checked sample contains the README-linked report Markdown, dashboard, `metadata/ai_usage.json`, `metadata/company_understanding.json`, `metadata/research_blueprint.json`, and `self_review/ai_self_review.md`.

## Legacy Boundary

Older Python commands are preserved only as compatibility notes:

- `openbb-research`
- `cresearch`
- `python scripts/company_research_tool.py`

They are not used in the main Quick Start.

## README UX

PASS

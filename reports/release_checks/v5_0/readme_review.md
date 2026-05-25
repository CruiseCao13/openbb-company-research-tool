# README Review

## Scope

Reviewed `README.md` after the v5.0 product-surface rewrite.

## Checklist

| Check | Status | Evidence |
|---|---|---|
| Introduces v5.0 as the current product | PASS | README title: `OpenBB Company Research Tool v5.0`; v5 pipeline shown explicitly. |
| Bilingual English/Chinese surface | PASS | English and Chinese descriptions, limitations, roadmap, and disclaimer are present. |
| Explains v5 vs v4 difference | PASS | README contrasts `v4.x: Data -> Rule-based profile -> Template report -> AI patch` with the v5 AI-led pipeline. |
| Responsibility split is clear | PASS | Rust, Python, AI, and Validator responsibilities are listed in a table. |
| US and China A-share support are described | PASS | README lists US/global and CN A-share providers plus `600519.SH`, `000001.SZ`, `300750.SZ` ticker examples. |
| US + CN sample links are present | PASS | README links AAPL, GOOGL, CAT, AMD, 600519.SH, and 000001.SZ samples. |
| Sample links exist | PASS | Required report, dashboard, PDF, company understanding, blueprint, ai_usage, and self-review files exist under `reports/samples/`. |
| External AI usage is explicit | PASS | README explains `metadata/ai_usage.json`, `external_ai_used`, `local_mock_used`, `new_external_ai_calls`, cache hits, and hard gate flags. |
| Dashboard/PDF/charts are described | PASS | README includes a dashboard/PDF/charts section and explains visual lint. |
| Content quality system is described | PASS | README lists content quality score, evidence map, data inventory, visual lint, chart/table quality, AI self-review, and training cases. |
| Limitations are honest | PASS | README states no investment advice, no target price, incomplete provider coverage, AI fallibility, and local fallback boundaries. |
| Old v4.3 current-status residue removed from README | PASS | README does not present v4.3/v4.4 as current product state. |

## Sample Link Evidence

Checked these sample families:

- `reports/samples/AAPL/`
- `reports/samples/GOOGL/`
- `reports/samples/CAT/`
- `reports/samples/AMD/`
- `reports/samples/600519.SH/`
- `reports/samples/000001.SZ/`

Each contains the README-linked report Markdown, dashboard, PDF, `metadata/company_understanding.json`, `metadata/research_blueprint.json`, `metadata/ai_usage.json`, and `self_review/ai_self_review.md`.

## Notes

Some samples intentionally use local fallback analysis. README tells users to inspect `metadata/ai_usage.json` and does not claim those samples are full external OpenAI analyses.

## README UX

PASS

# v5 Sample Link Audit

Generated: 2026-05-25 Asia/Singapore

## Scope

Checked README-linked v5 samples:

- `AAPL`
- `GOOGL`
- `CAT`
- `AMD`
- `600519.SH`
- `000001.SZ`

Each sample must contain:

- report Markdown
- `dashboard.html`
- `metadata/ai_usage.json`
- `metadata/company_understanding.json`
- `metadata/research_blueprint.json`
- `self_review/ai_self_review.md`

## Results

| Ticker | Market | Report | Dashboard | AI Usage | Understanding | Blueprint | Self Review | AI Source Label |
|---|---|---|---|---|---|---|---|---|
| AAPL | US | PASS | PASS | PASS | PASS | PASS | PASS | local fallback sample |
| GOOGL | US | PASS | PASS | PASS | PASS | PASS | PASS | local fallback sample |
| CAT | US | PASS | PASS | PASS | PASS | PASS | PASS | local fallback sample |
| AMD | US | PASS | PASS | PASS | PASS | PASS | PASS | local fallback sample |
| 600519.SH | CN A-share | PASS | PASS | PASS | PASS | PASS | PASS | local fallback sample |
| 000001.SZ | CN A-share | PASS | PASS | PASS | PASS | PASS | PASS | local fallback sample |

## AI Usage Summary

All checked samples are clearly labelled local fallback samples in README. Their `metadata/ai_usage.json` files show:

- `external_ai_used=false`
- `local_mock_used=true`
- `new_external_ai_calls=0`
- `cache_hits=0`
- `model=local-compact-analyst-fallback`
- `tasks=4`

## Result

README links are real v5 sample artifacts and do not claim external OpenAI usage for local fallback samples.

Status: PASS

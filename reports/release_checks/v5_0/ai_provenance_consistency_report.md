# v5 AI Provenance Consistency Report

Generated: 2026-05-25 Asia/Singapore

## Scope

Searched README, docs, Rust crates, release checks, and sample outputs for AI-source wording and provenance fields:

- `external_ai_used`
- `local_mock_used`
- `new_external_ai_calls`
- `local fallback`
- `local mock`
- `external_openai`
- `ai_usage`
- ambiguous wording such as `AI completed`, `AI reviewed`, and `AI layer`

## Findings

| Area | Result | Notes |
|---|---|---|
| README | PASS | Explains `metadata/ai_usage.json` as the source of truth and labels all samples as local fallback. |
| Sample reports | PASS | Top status blocks display AI source, external usage, local mock usage, calls, cache hits, model, and prompt versions. |
| Sample dashboards | PASS | AI Source cards show local fallback warnings where applicable. |
| Sample AI JSON | PASS | AI artifacts include `ai_provenance`; `ai_usage.json` records aggregate task provenance. |
| Batch summary code | PASS | Aggregates external AI calls, local fallback reports, cache hits, and skipped AI reports from run usage. |
| Codex self-review | PASS | States whether external OpenAI was used and cites artifact paths. |
| Historical docs | ACCEPTED | Older references to an AI layer are historical or legacy notes, not current v5 claims. |

## Sample Provenance Counts

| Ticker | external_ai_used | local_mock_used | new_external_ai_calls | cache_hits | model | tasks |
|---|---|---|---:|---:|---|---:|
| AAPL | false | true | 0 | 0 | local-compact-analyst-fallback | 4 |
| GOOGL | false | true | 0 | 0 | local-compact-analyst-fallback | 4 |
| CAT | false | true | 0 | 0 | local-compact-analyst-fallback | 4 |
| AMD | false | true | 0 | 0 | local-compact-analyst-fallback | 4 |
| 600519.SH | false | true | 0 | 0 | local-compact-analyst-fallback | 4 |
| 000001.SZ | false | true | 0 | 0 | local-compact-analyst-fallback | 4 |

## Result

No sample or README claim presents local/mock output as real external OpenAI analysis. Real external API use must be verified with `metadata/ai_usage.json`.

Status: PASS

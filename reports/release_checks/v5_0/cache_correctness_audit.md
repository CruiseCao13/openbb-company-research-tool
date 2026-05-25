# Cache Correctness Audit

Generated: 2026-05-25 Asia/Singapore

## Findings

| Area | Status | Evidence |
|---|---|---|
| AI cache root anchored | PASS | `research_core::paths::ai_cache_dir()` resolves to repo-root `reports/_cache/ai`. |
| `--no-ai-cache` external run | PASS | `api_verify_aapl_real` has `cache_hits=0` and `new_external_ai_calls=4`. |
| Batch cache stats surfaced | PASS | batch summary includes provider/AI cache fields and local fallback count. |
| `research-rs/reports` cache leak | PASS | Directory absent after path audit and batch audit. |
| Provider digest invalidation | WARNING | Cache key tests exist, but this audit did not mutate provider payloads to prove invalidation end-to-end. |

Status: WARNING

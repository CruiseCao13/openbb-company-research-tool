# RKLB External Manual Verification Policy

Generated: 2026-05-25

This autopilot pass did not run a new external API call. External verification is intentionally manual/gated and not required by CI.

Existing checked run:

- `reports/RKLB/runs/manual_verify_rklb_real`

Existing `ai_usage.json` shows:

- `external_ai_used=true`
- `local_mock_used=false`
- `new_external_ai_calls=4`
- `cache_hits=0`
- `model=gpt-4.1-mini`

Frame:

- Speculative Aerospace / Space Systems

The run is acceptable as historical manual evidence, but it was not rerun during CI stabilization.

Status: SKIPPED for new external call; existing evidence present.

# Quality Judge Provenance Check

- Path: `reports/training_runs/training_fixture_sanity_lunr/quality_judge_provenance.json`
- Exists: True

```json
{
  "cache_hit": false,
  "external_ai_used": false,
  "local_mock_used": true,
  "model": "local-deterministic-quality-layer",
  "new_external_ai_call": false,
  "note": "Quality score is from deterministic local training layer, not an external AI judge.",
  "prompt_version": "content_quality_judge_v1",
  "source": "local_deterministic_quality_layer"
}
```

Final status: PASS - local deterministic judge is clearly marked and not represented as external AI.

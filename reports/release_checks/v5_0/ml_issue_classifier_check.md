# ML Issue Classifier / Similar Failure Retrieval Check

- Retrieval report: `reports/training_runs/training_fixture_sanity_lunr/similar_failure_retrieval.md`
- Exists: True
- Baseline classifier source: deterministic issue classifier in `research-rs/crates/research-batch/src/training.rs`
- Current issue distribution:

```text
# Issue Distribution

| Issue type | Count | Fix target |
|---|---:|---|
| local_mock_case | 1 | prompt |

```

Final status: WARNING - baseline issue classifier exists; similarity retrieval is issue-cluster based, not a trained statistical model yet.

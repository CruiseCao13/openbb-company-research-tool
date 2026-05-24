# v5.0 Engine Design

v5.0 introduces a separate Rust-powered research engine.

The intended responsibility split is:

- Rust: orchestration, run folders, validation, reporting, batch evaluation, and
  packaging.
- Python: data-source adapters and provider normalization.
- AI layer: company understanding, financial interpretation, research blueprint,
  and self-review.
- Validator: fact boundaries, unsupported claims, and human-review status.

The current implementation uses a local compact analyst fallback. It is not a
paid external AI call and does not pretend to be one. This keeps tests and batch
evaluation repeatable while the schema and run-folder contract settle.

## Commands

```bash
research-rs run AAPL --market us --provider auto --ai compact --run-id v5_aapl_validation --pack --force
research-rs run 600519.SH --market cn --provider auto --ai compact --run-id v5_600519_validation --pack --force
research-rs batch eval_sets/broad_30_probe.yaml --workers 2 --ai compact --run-id v5_broad_30_validation_clean --pack --force
```

## Honest Boundaries

- No buy/sell/hold recommendations.
- No target prices.
- No fabricated segment, pipeline, foundry, or regulatory data.
- Provider failures produce explicit fallback status.
- External AI calls are reported separately from local compact review.


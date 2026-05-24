# Batch Evaluation Workflow

v4.4 adds a batch evaluation foundation for cross-industry regression and local system-training cases.

This is not model fine-tuning. The goal is to let code test report quality broadly, then use compact review only for failed or suspicious cases.

## Recommended Order

Run the smoke set first:

```bash
openbb-research batch eval_sets/smoke_12.yaml --both --full --pack --max-workers 2 --no-ai
openbb-research batch eval_sets/smoke_12.yaml --ai-review-failures --max-ai-reviews 5 --resume
```

Only after `smoke_12` is stable, run the broad set deterministically:

```bash
openbb-research batch eval_sets/broad_200.yaml --both --full --pack --max-workers 2 --no-ai
```

Then review only failures:

```bash
openbb-research batch eval_sets/broad_200.yaml --ai-review-failures --max-ai-reviews 40 --resume
```

Do not run `broad_200` with full AI review by default.

## Output Files

Batch outputs are written to:

```text
reports/batch_runs/<batch_id>/
```

Key files:

- `batch_summary.md`: human-readable dashboard
- `failures.md`: failure details and suggested fixes
- `warnings.md`: warning-level report review
- `profile_distribution.md`: asset-profile coverage
- `failure_type_distribution.md`: repeated failure modes
- `training_cases_generated.jsonl`: local system-training cases
- `credit_usage_estimate.md`: compact review and credit usage
- `ai_review_summary.md`: compact review scope

## Credit Control

Batch mode uses deterministic checks first.

External paid AI calls are not made by the local compact review path. The compact review payload excludes full reports, CSV files, and charts.

## What Counts As Success

The batch system is useful when it:

- isolates provider/data/report failures by ticker
- keeps the whole batch running after one failure
- exposes repeated failure types
- generates actionable training cases
- avoids broad AI review by default
- makes the summary readable enough that the user does not need to open 200 reports manually

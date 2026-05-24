# Training Cases

This folder stores local system-training cases generated from deterministic batch failures, warning patterns, and compact failure reviews.

These files are not model fine-tuning data and do not call fine-tune APIs. They are local regression artifacts used to make future code, routing, report templates, and tests smarter.

Typical flow:

1. Run `openbb-research batch eval_sets/smoke_12.yaml --both --full --pack --no-ai`.
2. Run `openbb-research batch eval_sets/smoke_12.yaml --ai-review-failures --max-ai-reviews 5 --resume`.
3. Review `reports/batch_runs/<batch_id>/failures.md`.
4. Convert repeated failure modes into deterministic tests and code fixes.

Generated files:

- `generated/*.yaml`: one case per ticker/date
- `corrections/correction_cases.jsonl`: append-only local correction case log

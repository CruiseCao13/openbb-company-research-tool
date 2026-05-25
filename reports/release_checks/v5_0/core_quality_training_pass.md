# v5 Core Quality Training Pass

Generated: 2026-05-25

## Scope

This pass paused dashboard/PDF/UI work and checked the core report-quality training loop: eval sets, hard regression cases, issue taxonomy, investment rubric, training CLI, training artifacts, quality matrix, training cases, ML features, and budget-controlled dry runs.

## Training System Status

Status: WARNING

The training system exists and runs end-to-end in local/budget-0 mode. It generates quality matrices, issue distributions, training cases, regression cases, prompt/validator suggestions, quality trend, cost report, model improvement review, self-repair plan/diff, and ML feature rows.

The warning is deliberate: this pass did not run external OpenAI training calls. Local/mock outputs were clearly labeled as `local_mock_case` and were not treated as external or positive cases.

## Eval Set Status

| Eval set | Groups | Tickers | Unique | Required hard cases |
| --- | ---: | ---: | ---: | --- |
| `eval_sets/broad_500_us_cn.yaml` | 34 | 500 | 500 | PASS |
| `eval_sets/regression_hard_cases.yaml` | 13 | 18 | 18 | PASS |

Required cases present: LUNR, RKLB, GOOGL, META, CAT, ISRG, LLY, AMD, T, ASTS, ZIM, INTC, JPM, AAPL, 600519.SH, 000001.SZ, 300750.SZ.

## Investment Rubric

Status: PASS

Added user-owned rubric library under `docs/investment_rubric/`:

- business model analysis
- financial statement analysis
- money flow analysis
- valuation method fit
- sector-specific analysis
- red flags
- data gaps
- unsupported claims
- A-share accounting rules
- report language style

## Commands Run

Regression hard cases:

```bash
cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- train eval_sets/regression_hard_cases.yaml \
  --stage regression \
  --workers 2 \
  --ai local \
  --budget-calls 0 \
  --max-iterations 1 \
  --run-id core_quality_regression_local \
  --force
```

Broad 500 dry run:

```bash
cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- train eval_sets/broad_500_us_cn.yaml \
  --stage broad_500 \
  --workers 2 \
  --ai local \
  --limit 10 \
  --offset 0 \
  --budget-calls 0 \
  --max-iterations 1 \
  --run-id core_quality_broad_500_dryrun_10 \
  --force
```

## Training Results

| Run | Reports scored | Average quality | Hard failures | Training cases | External calls | Budget remaining |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `reports/training_runs/core_quality_regression_local` | 18 | 78.0 | 0 | 18 | 0 | 0 |
| `reports/training_runs/core_quality_broad_500_dryrun_10` | 10 | 78.0 | 0 | 10 | 0 | 0 |

Top issue types:

- `local_mock_case`: 18 in regression hard cases
- `local_mock_case`: 10 in broad_500 dry run

This is expected for local/budget-0 verification and prevents local/mock output from entering positive external training.

## Output Artifacts

Regression run:

- `reports/training_runs/core_quality_regression_local/quality_matrix.csv`
- `reports/training_runs/core_quality_regression_local/quality_matrix.json`
- `reports/training_runs/core_quality_regression_local/issue_distribution.md`
- `reports/training_runs/core_quality_regression_local/training_cases_generated.jsonl`
- `reports/training_runs/core_quality_regression_local/regression_cases_generated.jsonl`
- `reports/training_runs/core_quality_regression_local/quality_score_trend.md`
- `reports/training_runs/core_quality_regression_local/prompt_improvement_suggestions.md`
- `reports/training_runs/core_quality_regression_local/validator_improvement_suggestions.md`
- `reports/training_runs/core_quality_regression_local/model_improvement_review.md`
- `reports/training_runs/core_quality_regression_local/cost_report.md`

Broad 500 dry run:

- `reports/training_runs/core_quality_broad_500_dryrun_10/quality_matrix.csv`
- `reports/training_runs/core_quality_broad_500_dryrun_10/quality_matrix.json`
- `reports/training_runs/core_quality_broad_500_dryrun_10/issue_distribution.md`
- `reports/training_runs/core_quality_broad_500_dryrun_10/training_cases_generated.jsonl`
- `reports/training_runs/core_quality_broad_500_dryrun_10/regression_cases_generated.jsonl`
- `reports/training_runs/core_quality_broad_500_dryrun_10/quality_score_trend.md`
- `reports/training_runs/core_quality_broad_500_dryrun_10/prompt_improvement_suggestions.md`
- `reports/training_runs/core_quality_broad_500_dryrun_10/validator_improvement_suggestions.md`
- `reports/training_runs/core_quality_broad_500_dryrun_10/model_improvement_review.md`
- `reports/training_runs/core_quality_broad_500_dryrun_10/cost_report.md`

ML features:

- `training/ml_features/report_features.jsonl`

Generated training-case store:

- `training/cases/generated/core_quality_broad_500_dryrun_10.jsonl`

Prompt change log:

- `training/prompt_versions/prompt_change_log.md`

Validator change log:

- `training/validator_versions/validator_change_log.md`

Issue taxonomy:

- `training/issue_taxonomy/issue_types.yaml`

## Current Blockers

- External staged training was not run in this pass; the next quality stage should run `regression_hard_cases` with `--ai compact --require-external-ai --budget-calls N` before claiming real external training improvement.
- Current quality issue distribution is dominated by `local_mock_case`, which is correct for a local dry run but not sufficient for external quality acceptance.
- ML feature extraction is a baseline JSONL feature export, not a trained model or fine-tune. Fine-tuning remains deferred.

## Final Status

Final status: WARNING

The training loop is real and runnable, broad_500 is present with 500 unique companies, regression hard cases now include RKLB and 300750.SZ, local/mock results are not mislabeled as external, and artifacts are generated. External AI training quality improvement remains the next gated step.

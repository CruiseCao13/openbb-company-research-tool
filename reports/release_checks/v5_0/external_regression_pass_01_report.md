# External Regression Training Pass 01

Generated: 2026-05-25

## Command

```bash
RESEARCH_ENGINE_ROOT="/Users/cruise/Projects/investment-tools/openbb-company-research-tool" \
cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- train eval_sets/external_regression_pass_01.yaml \
  --stage regression \
  --workers 2 \
  --ai compact \
  --require-external-ai \
  --no-ai-cache \
  --budget-calls 120 \
  --max-iterations 2 \
  --run-id external_regression_pass_01
```

The shell did not expose `OPENAI_API_KEY`, but repo-root `.env` contained it and the Rust CLI loaded it without printing the key.

## Companies

LUNR, RKLB, GOOGL, CAT, ISRG, JPM, 600519.SH, 000001.SZ.

## API Provenance

| Metric | Value |
| --- | ---: |
| Final-run external AI calls | 32 |
| Total external calls used during corrective attempts | 96 |
| Final-run cache hits | 0 |
| Final-run local mock reports | 0 |
| Budget calls | 120 |
| Cumulative budget remaining | 24 |

Every final per-ticker `metadata/ai_usage.json` has `external_ai_used=true`, `local_mock_used=false`, `new_external_ai_calls=4`, and `cache_hits=0`.

## Final Frames

| Ticker | Frame | Status | Provider status |
| --- | --- | --- | --- |
| LUNR | Speculative Aerospace / Space Systems | WARNING | PASS |
| RKLB | Speculative Aerospace / Space Systems | WARNING | PASS |
| GOOGL | Platform Internet / Digital Ads / Cloud | WARNING | PASS |
| CAT | Cyclical / Industrial Cycle | WARNING | PASS |
| ISRG | Medical Devices / Surgical Robotics | WARNING | PASS |
| JPM | Financials / Bank-like Screening | WARNING | PASS |
| 600519.SH | Unknown / Data-Limited Screening | WARNING | PROVIDER_ERROR |
| 000001.SZ | Financials / Bank-like Screening | WARNING | PROVIDER_ERROR |

ISRG initially exposed a framework-gate weakness and was corrected before the final pass. The final ISRG frame is Medical Devices / Surgical Robotics, not biotech.

## Quality

| Metric | Value |
| --- | ---: |
| Reports scored | 8 |
| Average quality score | 78.0 |
| Hard failures | 0 |
| Wrong framework cases | 0 |
| Weak money flow cases | 0 |
| Provider data gaps | 2 |
| Training cases generated | 8 |
| External correction cases generated | 8 |
| Negative regression cases generated | 0 |

Issue distribution: 6 `weak_company_understanding`, 2 `provider_data_gap`.

## Hard-Rule Checks

- LUNR: no telecom, wireless, broadband, subscriber churn, or regulated telecom terms in positive analysis.
- RKLB: no telecom, wireless, broadband, subscriber churn, bank, or insurance frame in positive analysis.
- GOOGL: no financials/bank/insurance frame.
- CAT: no insurance/bank frame.
- ISRG: biotech terms appear only in `not_this` / `must_not_analyze_as_core`, not as the selected frame.
- JPM and 000001.SZ: industrial FCF / net debt terms appear only in `must_not_analyze_as_core`.
- 600519.SH: provider adapter returned data-limited payload, so the report is WARNING and not a full A-share consumer analysis.

## Artifacts

- Training root: `reports/training_runs/external_regression_pass_01/`
- Quality matrix: `reports/training_runs/external_regression_pass_01/quality_matrix.csv`
- Cost report: `reports/training_runs/external_regression_pass_01/cost_report.md`
- Training cases: `reports/training_runs/external_regression_pass_01/training_cases_generated.jsonl`
- External correction cases: `reports/training_runs/external_regression_pass_01/external_correction_cases_generated.jsonl`
- Negative regression cases: `reports/training_runs/external_regression_pass_01/negative_regression_cases_generated.jsonl`
- Final acceptance: `reports/training_runs/external_regression_pass_01/final_acceptance.md`

## Final Status

Status: WARNING

Reason: the external no-cache regression pass completed with real OpenAI calls and no local/mock contamination, but both CN A-share tickers hit provider-data-gap conditions in this environment. They are classified as provider gaps, not AI failures.

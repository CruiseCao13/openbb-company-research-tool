# Training Case Hygiene Report

Generated: 2026-05-25

## Files Checked

- `training_cases/corrections/v5_external_correction_cases.jsonl`
- `training_cases/corrections/v5_local_mock_cases.jsonl`
- `training_cases/corrections/v5_negative_regression_cases.jsonl`

## Counts

| File | Records |
| --- | ---: |
| external correction cases | 0 |
| local mock cases | 46 |
| negative regression cases | 0 |

## Validation

| Check | Result |
| --- | --- |
| non-ticker strings in `ticker` field | PASS |
| frame names in `ticker` field | PASS |
| forbidden terms in `ticker` field | PASS |
| local/mock cases in external correction file | PASS |
| local/mock positive external cases | PASS |
| old polluted `v5_correction_cases.jsonl` removed | PASS |

Rejected polluted records from old file: 17

External cases are empty because no external training run was performed in this autopilot pass. This is acceptable and must not be described as completed external training.

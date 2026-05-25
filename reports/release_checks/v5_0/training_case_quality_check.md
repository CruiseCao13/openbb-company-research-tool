# Training Case Quality Check

- Source path: `reports/training_runs/training_fixture_sanity_lunr/training_cases_generated.jsonl`
- Cases: 1
- Required fields missing: none
- Local/mock positive cases: 0
- wrong_profile cases: 0
- hallucinated_revenue_engine cases: 0

Final status: PASS

## LUNR iterative failure cases

- Path: `reports/training_runs/lunr_iterative_fix/training_cases_generated.jsonl`
- Cases: 6
- Issue types: wrong_profile, hallucinated_revenue_engine, weak_money_flow, generic_chart_explanation, report_status_wrong, self_review_failed_to_catch
- Source: historical failed external run `reports/LUNR/runs/api_verify_lunr_real`
- External fixed run verified: False

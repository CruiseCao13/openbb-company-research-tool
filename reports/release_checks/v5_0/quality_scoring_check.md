# Quality Scoring Check

- Matrix JSON: `reports/training_runs/training_fixture_sanity_lunr/quality_matrix.json`
- Rows: 1
- Missing dimensions: none
- wrong_framework cap <= 59 rule satisfied in observed rows: True

## Observed scores

```json
[
  {
    "ticker": "LUNR",
    "run_folder": "/Users/cruise/Projects/investment-tools/openbb-company-research-tool/reports/LUNR/runs/training_fixture_sanity_lunr_batch_LUNR",
    "quality_score": 78,
    "grade": "ACCEPTABLE",
    "company_understanding_score": 13,
    "business_model_score": 8,
    "financial_interpretation_score": 13,
    "money_flow_score": 13,
    "blueprint_fit_score": 8,
    "valuation_fit_score": 6,
    "risk_score": 6,
    "data_gap_score": 6,
    "chart_table_score": 5,
    "language_score": 5,
    "unsupported_claims_score": 0,
    "hard_failures": [],
    "specific_issues": [
      "report_status requires human review; quality score capped"
    ],
    "rewrite_required_sections": [],
    "training_case_type": "local_mock_case",
    "human_review_required": true
  }
]
```

Final status: PASS

# v5 AI Training System Verification

## 1. Training CLI status

- Report: `training_cli_check.md`
- Status: PASS. `research-rs train` exists and exposes staged training options.

## 2. broad_500 eval set status

- Report: `broad_500_eval_set_check.md`
- Status: PASS. 500 unique tickers, 250 US and 250 CN A-share.

## 3. Regression hard cases status

- Setup report: `regression_hard_cases_setup.md`
- Run report: `regression_hard_cases_check.md`
- Status: PASS. Hard cases include LUNR, AAPL, GOOGL, CAT, AMD, T, ZIM, JPM, 600519.SH, and 000001.SZ.

## 4. LUNR training result

- Historical failure source: `reports/LUNR/runs/api_verify_lunr_real`
- Negative training cases: `reports/training_runs/lunr_iterative_fix/training_cases_generated.jsonl`
- External fixed run attempted: yes
- External fixed run completed: no
- Blocker: `OPENAI_API_KEY` is not visible in the Codex tool shell.

## 5. LUNR final frame

- Historical failed frame: `Telecom / Infrastructure Cash Flow`
- Local guard fixed frame: `Speculative Aerospace / Space Systems`
- External fixed frame: not verified in this shell.

## 6. Forbidden telecom terms check

- Historical failed forbidden terms: ['Telecom / Infrastructure Cash Flow', 'wireless service revenue', 'broadband / network revenue']
- Local regression LUNR forbidden terms: PASS; see `regression_hard_cases_check.md`.

## 7. Content quality scoring result

- Report: `quality_scoring_check.md`
- Status: PASS for local fixture quality dimensions. Historical failed external LUNR is capped as FAIL in `lunr_iterative_fix/quality_score.json`.

## 8. Training cases generated

- Local fixture cases: `reports/training_runs/training_fixture_sanity_lunr/training_cases_generated.jsonl`
- LUNR failure cases: `reports/training_runs/lunr_iterative_fix/training_cases_generated.jsonl`
- LUNR issue types: wrong_profile, hallucinated_revenue_engine, weak_money_flow, generic_chart_explanation, report_status_wrong, self_review_failed_to_catch

## 9. Issue taxonomy status

- Report: `issue_taxonomy_check.md`
- Status: PASS.

## 10. ML feature extraction status

- Report: `ml_feature_extraction_check.md`
- Path: `training/ml_features/report_features.jsonl`
- Status: PASS for baseline feature extraction.

## 11. Prompt training status

- Report: `prompt_training_check.md`
- Path: `training/prompt_versions/prompt_change_log.md`

## 12. Validator training status

- Report: `validator_training_check.md`
- Path: `training/validator_versions/validator_change_log.md`

## 13. Compact payload training status

- Report: `compact_payload_training_check.md`
- Path: `training/cases/compact_payload_cases.jsonl`

## 14. Self-repair loop status

- Report: `self_repair_loop_check.md`
- Status: WARNING. The loop writes plan/diff/log artifacts; source patches were applied manually by this pass rather than automatically by the loop.

## 15. Anti-overfitting result

- Report: `anti_overfitting_check.md`
- Status: PASS for local regression.

## 16. broad_500 dryrun result

- Report: `broad_500_dryrun_check.md`
- Training output: `reports/training_runs/broad_500_dryrun_10`
- Status: PASS. 10 tickers, no external AI calls, budget respected.

## 17. Cost control result

- Report: `training_cost_control_check.md`
- Cost path: `reports/training_runs/broad_500_dryrun_10/cost_report.md`
- Status: PASS for local budget dry run.

## 18. Build and Secret Scan

- `cargo fmt --all`: PASS
- `cargo test`: PASS, 108 tests passed
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- `git diff --check`: PASS
- Secret scan: PASS, no full `sk-...` key detected; see `training_secret_scan_check.md`.

## 19. Remaining blockers

- External fixed LUNR iteration cannot be completed until a rotated `OPENAI_API_KEY` is visible to the Codex tool shell.
- Similar-failure retrieval is a deterministic issue-cluster baseline, not a trained statistical model.

## 20. Final status

Final status: FAIL

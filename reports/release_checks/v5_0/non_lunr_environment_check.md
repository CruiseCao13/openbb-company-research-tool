# Non-LUNR Environment Check

## Environment
```text
v5-rust-ai-blueprint
 M research-rs/crates/research-ai/prompts/company_understanding_v1.md
 M research-rs/crates/research-ai/prompts/self_review_v1.md
 M research-rs/crates/research-ai/src/client.rs
 M research-rs/crates/research-ai/src/company_understanding.rs
 M research-rs/crates/research-ai/src/financial_interpretation.rs
 M research-rs/crates/research-ai/src/research_blueprint.rs
 M research-rs/crates/research-ai/src/self_review.rs
 M research-rs/crates/research-ai/src/tests.rs
 M research-rs/crates/research-batch/src/eval_set.rs
 M research-rs/crates/research-batch/src/lib.rs
 M research-rs/crates/research-batch/src/quality.rs
 M research-rs/crates/research-batch/src/runner.rs
 M research-rs/crates/research-batch/src/tests.rs
 M research-rs/crates/research-cli/src/main.rs
 M research-rs/crates/research-core/src/paths.rs
 M research-rs/crates/research-core/src/pipeline.rs
 M research-rs/crates/research-core/src/tests.rs
 M research-rs/crates/research-core/src/validation.rs
 M research-rs/crates/research-report/src/markdown.rs
 M research-rs/crates/research-report/src/renderer.rs
 M research-rs/crates/research-report/src/tests.rs
 M training_cases/corrections/v5_correction_cases.jsonl
?? eval_sets/regression_hard_cases.yaml
?? reports/release_checks/v5_0/ai_training_system_verification.md
?? reports/release_checks/v5_0/anti_overfitting_check.md
?? reports/release_checks/v5_0/broad_500_dryrun_check.md
?? reports/release_checks/v5_0/broad_500_eval_set_check.md
?? reports/release_checks/v5_0/compact_payload_training_check.md
?? reports/release_checks/v5_0/dotenv_external_ai_verification.md
?? reports/release_checks/v5_0/issue_taxonomy_check.md
?? reports/release_checks/v5_0/lunr_training_external_api_check.md
?? reports/release_checks/v5_0/ml_feature_extraction_check.md
?? reports/release_checks/v5_0/ml_issue_classifier_check.md
?? reports/release_checks/v5_0/non_lunr_environment_check.md
?? reports/release_checks/v5_0/prompt_training_check.md
?? reports/release_checks/v5_0/quality_judge_provenance_check.md
?? reports/release_checks/v5_0/quality_scoring_check.md
?? reports/release_checks/v5_0/regression_hard_cases_check.md
?? reports/release_checks/v5_0/regression_hard_cases_setup.md
?? reports/release_checks/v5_0/self_repair_loop_check.md
?? reports/release_checks/v5_0/training_case_quality_check.md
?? reports/release_checks/v5_0/training_cli_check.md
?? reports/release_checks/v5_0/training_cost_control_check.md
?? reports/release_checks/v5_0/training_secret_scan_check.md
?? reports/release_checks/v5_0/training_system_environment_check.md
?? reports/release_checks/v5_0/training_system_precheck.md
?? reports/release_checks/v5_0/training_system_structure_check.md
?? reports/release_checks/v5_0/validator_training_check.md
?? research-rs/crates/research-batch/src/training.rs
?? training/
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
rustc 1.95.0 (59807616e 2026-04-14)
OPENAI_API_KEY set? no
```

## Secret Scan
- Raw matches: 26
- Full OpenAI key pattern found: no

```text
./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./research-rs/crates/research-cli/src/main.rs:866:        fs::write(root.join(".env"), "OPENAI_API_KEY=test-dotenv-key\n").unwrap();
./research-rs/crates/research-cli/src/main.rs:883:        fs::write(root.join(".env"), "OPENAI_API_KEY=test-dotenv-key\n").unwrap();
./research-rs/crates/research-cli/src/main.rs:902:        fs::write(root.join(".env"), "OPENAI_API_KEY='test-from-root-key'\n").unwrap();
./research-rs/crates/research-cli/src/main.rs:937:        fs::write(root.join(".env"), "OPENAI_API_KEY=test-secret-value\n").unwrap();
./reports/release_checks/v5_0/training_secret_scan_check.md:8:./reports/release_checks/v5_0/secret_safety_report.md:8:rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" . -g '!target/**' -g '!.venv/**' -g '!reports/_cache/**' -g '!*.lock'
./reports/release_checks/v5_0/training_secret_scan_check.md:9:./reports/release_checks/v5_0/secret_safety_report.md:14:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/training_secret_scan_check.md:10:./reports/release_checks/v5_0/secret_safety_report.md:15:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/training_secret_scan_check.md:11:./reports/release_checks/v5_0/secret_safety_report.md:24:- `OPENAI_API_KEY="your_key"` in README is a placeholder, not a secret.
./reports/release_checks/v5_0/training_secret_scan_check.md:12:./reports/release_checks/v5_0/secret_safety_report.md:25:- `.env.example` intentionally contains `OPENAI_API_KEY=` with an empty value.
./reports/release_checks/v5_0/training_secret_scan_check.md:13:./reports/release_checks/v5_0/training_secret_scan_check.md:3:- Command: `rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" ...`
./reports/release_checks/v5_0/training_secret_scan_check.md:14:./reports/release_checks/v5_0/training_secret_scan_check.md:9:./reports/release_checks/v5_0/secret_safety_report.md:8:rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" . -g '!target/**' -g '!.venv/**' -g '!reports/_cache/**' -g '!*.lock'
./reports/release_checks/v5_0/training_secret_scan_check.md:15:./reports/release_checks/v5_0/training_secret_scan_check.md:10:./reports/release_checks/v5_0/secret_safety_report.md:14:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/training_secret_scan_check.md:16:./reports/release_checks/v5_0/training_secret_scan_check.md:11:./reports/release_checks/v5_0/secret_safety_report.md:15:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/training_secret_scan_check.md:17:./reports/release_checks/v5_0/training_secret_scan_check.md:14:./reports/release_checks/v5_0/secret_safety_report.md:24:- `OPENAI_API_KEY="your_key"` in README is a placeholder, not a secret.
./reports/release_checks/v5_0/training_secret_scan_check.md:18:./reports/release_checks/v5_0/training_secret_scan_check.md:15:./reports/release_checks/v5_0/secret_safety_report.md:25:- `.env.example` intentionally contains `OPENAI_API_KEY=` with an empty value.
./reports/release_checks/v5_0/training_secret_scan_check.md:19:./reports/release_checks/v5_0/training_secret_scan_check.md:18:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/training_secret_scan_check.md:20:./reports/release_checks/v5_0/training_secret_scan_check.md:19:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/training_secret_scan_check.md:21:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/training_secret_scan_check.md:22:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/secret_safety_report.md:8:rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" . -g '!target/**' -g '!.venv/**' -g '!reports/_cache/**' -g '!*.lock'
./reports/release_checks/v5_0/secret_safety_report.md:14:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/secret_safety_report.md:15:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/secret_safety_report.md:24:- `OPENAI_API_KEY="your_key"` in README is a placeholder, not a secret.
./reports/release_checks/v5_0/secret_safety_report.md:25:- `.env.example` intentionally contains `OPENAI_API_KEY=` with an empty value.
./.env.example:2:OPENAI_API_KEY=
```

Final status: BLOCKED - OPENAI_API_KEY not visible in shell

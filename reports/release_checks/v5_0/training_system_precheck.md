# Training System Precheck

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
?? reports/release_checks/v5_0/issue_taxonomy_check.md
?? reports/release_checks/v5_0/lunr_training_external_api_check.md
?? reports/release_checks/v5_0/ml_feature_extraction_check.md
?? reports/release_checks/v5_0/ml_issue_classifier_check.md
?? reports/release_checks/v5_0/prompt_training_check.md
?? reports/release_checks/v5_0/quality_judge_provenance_check.md
?? reports/release_checks/v5_0/quality_scoring_check.md
?? reports/release_checks/v5_0/regression_hard_cases_check.md
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

## Training-related code
```text
research-rs/crates/research-ai/Cargo.toml
research-rs/crates/research-ai/prompts/chart_explanation_v1.md
research-rs/crates/research-ai/prompts/company_understanding_v1.md
research-rs/crates/research-ai/prompts/content_quality_judge_v1.md
research-rs/crates/research-ai/prompts/financial_interpretation_v1.md
research-rs/crates/research-ai/prompts/research_blueprint_v1.md
research-rs/crates/research-ai/prompts/self_review_v1.md
research-rs/crates/research-ai/prompts/table_explanation_v1.md
research-rs/crates/research-ai/src/prompts.rs
research-rs/crates/research-batch/Cargo.toml
research-rs/crates/research-batch/src/quality.rs
research-rs/crates/research-batch/src/training_case.rs
research-rs/crates/research-batch/src/training.rs
research-rs/crates/research-cli/Cargo.toml
research-rs/crates/research-core/Cargo.toml
research-rs/crates/research-report/Cargo.toml
```

## CLI help snapshots
See also: training_cli_check.md

Final status: WARNING - training system exists, but external API is not visible in this Codex shell.

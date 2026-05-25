# LUNR Training External API Check

- Command: `research-rs train ../eval_sets/regression_hard_cases.yaml --stage regression --ai compact --require-external-ai --no-ai-cache --max-iterations 3 --budget-calls 80 --only-wrong-framework --run-id lunr_training_verification`
- Exit code: 1
- OPENAI_API_KEY visible to shell: no

## stdout
```text
```
## stderr
```text
Error: OPENAI_API_KEY missing; --require-external-ai forbids local fallback. See docs/error_handbook.md#ai-json-invalid
```
## Training artifacts
No training artifacts were generated because the external-AI hard gate failed before batch execution.

Final status: FAIL - external API verification blocked or failed

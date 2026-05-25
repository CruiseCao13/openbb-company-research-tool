# Secret Scan Result

Generated: 2026-05-25

## Real Secret Scan

Command:

```bash
rg -n "sk-[A-Za-z0-9_-]{20,}|Authorization: Bearer sk-" . \
  -g '!target/**' \
  -g '!research-rs/target/**' \
  -g '!reports/release_checks/**' \
  -g '!reports/_cache/**' \
  -g '!reports/**/runs/**' \
  -g '!reports/batch_runs/**' \
  -g '!reports/training_runs/**' \
  -g '!.venv/**' \
  -g '!Cargo.lock' \
  -g '!.env'
```

Result: PASS, no real secret matches.

## Placeholder Scan

Placeholders are report-only and allowed in:

- `README.md`
- `.env.example`
- Rust test fixtures
- release check policy documents

Examples:

- `OPENAI_API_KEY=`
- `OPENAI_API_KEY="your_key"`
- `OPENAI_API_KEY=test-dotenv-key`
- `OPENAI_API_KEY=test-from-root-key`
- `OPENAI_API_KEY=test-secret-value`

Result: PASS, placeholders are not hard failures.

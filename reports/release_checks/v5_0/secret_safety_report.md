# Secret Safety Report

Generated: 2026-05-25 Asia/Singapore

## Scan Pattern

```bash
rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" . -g '!target/**' -g '!.venv/**' -g '!reports/_cache/**' -g '!*.lock'
```

## Matches

```text
./.env.example:2:OPENAI_API_KEY=
./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./tests/test_ai_review.py:83:    def test_missing_api_key_does_not_crash(self):
./docs/development_log.md:250:client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
```

## Judgment

The scan found placeholders and code/documentation references only:

- `OPENAI_API_KEY="your_key"` in README is a placeholder, not a secret.
- `.env.example` intentionally contains `OPENAI_API_KEY=` with an empty value.
- `docs/development_log.md` contains an example `os.getenv("OPENAI_API_KEY")`, not a key.
- No `sk-...` style full API key was found.

Status: PASS

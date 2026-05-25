# Training Verification Secret Scan

- Raw matches: 15
- Full OpenAI key pattern found: no

## Raw matches
```text
./reports/release_checks/v5_0/secret_safety_report.md:8:rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" . -g '!target/**' -g '!.venv/**' -g '!reports/_cache/**' -g '!*.lock'
./reports/release_checks/v5_0/secret_safety_report.md:14:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/secret_safety_report.md:15:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/secret_safety_report.md:24:- `OPENAI_API_KEY="your_key"` in README is a placeholder, not a secret.
./reports/release_checks/v5_0/secret_safety_report.md:25:- `.env.example` intentionally contains `OPENAI_API_KEY=` with an empty value.
./reports/release_checks/v5_0/training_secret_scan_check.md:3:- Command: `rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" ...`
./reports/release_checks/v5_0/training_secret_scan_check.md:9:./reports/release_checks/v5_0/secret_safety_report.md:8:rg -n "sk-[A-Za-z0-9_-]{20,}|OPENAI_API_KEY=|api_key|secret_key|Authorization: Bearer" . -g '!target/**' -g '!.venv/**' -g '!reports/_cache/**' -g '!*.lock'
./reports/release_checks/v5_0/training_secret_scan_check.md:10:./reports/release_checks/v5_0/secret_safety_report.md:14:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/training_secret_scan_check.md:11:./reports/release_checks/v5_0/secret_safety_report.md:15:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./reports/release_checks/v5_0/training_secret_scan_check.md:14:./reports/release_checks/v5_0/secret_safety_report.md:24:- `OPENAI_API_KEY="your_key"` in README is a placeholder, not a secret.
./reports/release_checks/v5_0/training_secret_scan_check.md:15:./reports/release_checks/v5_0/secret_safety_report.md:25:- `.env.example` intentionally contains `OPENAI_API_KEY=` with an empty value.
./reports/release_checks/v5_0/training_secret_scan_check.md:18:./.env.example:2:OPENAI_API_KEY=
./reports/release_checks/v5_0/training_secret_scan_check.md:19:./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./README.md:94:OPENAI_API_KEY="your_key" cargo run -p research-rs -- run AAPL \
./.env.example:2:OPENAI_API_KEY=
```

## Assessment
No full OpenAI API key was detected in the repository scan. Matches are placeholders, tests, docs, or audit text.

Final status: PASS

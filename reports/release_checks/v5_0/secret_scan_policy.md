# Secret Scan Policy

Generated: 2026-05-25

## Purpose

The v5 secret scan is split into two levels so real key leaks fail the release gate while placeholders and test fixtures remain visible but non-blocking.

## Level 1: Real Secret Scan

Status: blocking

Fail on:

- `sk-[A-Za-z0-9_-]{20,}`
- `Authorization: Bearer sk-`

These patterns represent real OpenAI-style key material or bearer usage and must not appear in source, README, reports, dashboards, packs, prompts, responses, or logs.

## Level 2: Placeholder Scan

Status: report-only

Report but do not fail on:

- `OPENAI_API_KEY=`
- `OPENAI_API_KEY="your_key"`
- `OPENAI_API_KEY='your_key'`
- `OPENAI_API_KEY=test-...`

Allowed locations:

- `README.md` placeholder examples
- `.env.example` empty placeholder
- Rust test fixtures using `test-dotenv-key`, `test-from-root-key`, or `test-secret-value`

## Recursive Report Exclusion

Default secret scans should exclude:

- `reports/release_checks/**`

Reason: release check reports quote earlier scan output. Including them in every default scan creates recursive noise and false-looking matches. Reports can still be explicitly audited by enabling report scanning.

## Current Policy Result

Real secret scan: PASS

Placeholder scan: report-only

Release check recursive scan: excluded by default

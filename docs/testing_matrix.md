# Testing Matrix

v4.3 tests focus on report logic rather than only string formatting.

## Required Coverage

- AAPL-like payload routes to Mature Compounder logic.
- RKLB-like payload routes to Speculative Growth logic.
- RKLB-like reports do not use mature-compounder valuation or Services / buyback language.
- Financial-like payloads avoid industrial FCF and net-debt / EBITDA as core interpretation.
- Cyclical-like payloads do not call low PE automatically cheap.
- Unknown companies degrade to Unknown / Data-Limited Screening.
- AI patch artifacts are generated and include old/new text.
- Locked blocks are not patched.
- Fallback usage is tracked.
- Organized report pack folders exist.

## Random Company Standard

The system does not need to perfectly classify every company.

It must:

- avoid hardcoded ticker behavior
- expose low framework confidence
- avoid unsupported valuation methods
- generate manual verification items
- suggest framework extensions when coverage is incomplete


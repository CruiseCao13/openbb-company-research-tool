# v5 Consistency Sweep

Generated: 2026-05-25 Asia/Singapore

## 1. What Was Scanned

- README product surface
- v5 and legacy docs
- examples
- Rust crates
- Python legacy scripts
- release-check reports
- v5 sample reports and dashboards
- sample AI provenance artifacts
- path anchoring helpers

## 2. Old v4/v3/v2 Remnants Found

Old-version references remain in historical docs, legacy Python implementation files, and legacy regression tests. These are allowed because they are explicitly not the v5 product surface.

Fixed items:

- README no longer contains old v4/v3/v2 product sections.
- `docs/cli_usage.md` is legacy-only.
- `docs/batch_evaluation.md` is marked as pre-v5 Python batch workflow.
- `docs/report_structure.md` is marked as pre-v5 report structure.
- `examples/example_commands.md` now leads with `research-rs` examples.

## 3. README Issues Fixed

README was rewritten as a v5 product homepage. It now covers:

- v5 AI-led Rust research engine positioning
- v4-to-v5 workflow change
- Rust/Python/AI/Validator responsibility split
- US and China A-share support
- Quick Start with `research-rs`
- real OpenAI API verification through `metadata/ai_usage.json`
- US and CN sample gallery
- output structure
- dashboard/PDF/charts
- content quality evaluation
- limitations, roadmap, and disclaimer
- short legacy pointer to `docs/history_v2_v4.md`

## 4. CLI Inconsistencies Fixed

The primary CLI in README and examples is now `research-rs`. Legacy commands remain only in legacy sections or compatibility implementation/tests.

Report: `reports/release_checks/v5_0/cli_consistency_report.md`

## 5. Sample Link Issues Fixed

README sample links were checked for AAPL, GOOGL, CAT, AMD, 600519.SH, and 000001.SZ. Required report, dashboard, `ai_usage`, company understanding, blueprint, and self-review files exist.

Report: `reports/release_checks/v5_0/sample_link_audit.md`

## 6. AI Provenance Issues Fixed

README and sample outputs clearly distinguish local fallback from real external OpenAI. Each sample is labelled local fallback and has `metadata/ai_usage.json` showing `external_ai_used=false`, `local_mock_used=true`, and `new_external_ai_calls=0`.

Report: `reports/release_checks/v5_0/ai_provenance_consistency_report.md`

## 7. Path Anchoring Issues Fixed

Generated output paths are routed through repo-root helpers for reports, AI cache, batch runs, quality runs, samples, and release checks. No `research-rs/reports` directory exists.

Report: `reports/release_checks/v5_0/path_anchoring_audit.md`

## 8. Report Structure Issues Fixed

v5 sample reports no longer contain old v4 markers such as `How to Read This Report`, `Research Score`, or old AI/language gate wording. They use the v5 AI-source/status, company identity, business model, money flow, financial interpretation, blueprint, risk, data-gap, self-review, chart/evidence, and locked-data appendix structure.

Report: `reports/release_checks/v5_0/report_structure_consistency_report.md`

## 9. Remaining Legacy Content and Why Allowed

- `scripts/`: retained Python workflow for compatibility.
- Python tests: retained regression coverage for legacy behavior.
- `docs/development_log.md`, `docs/known_issues_and_roadmap.md`, and `docs/history_v2_v4.md`: historical notes.
- `docs/v4_3_report_system_design.md`: archived design reference.

These files are not presented as the current v5 product entry point.

## 10. Tests Added

- README v5 title and bilingual sections
- README primary Quick Start uses `research-rs`
- legacy commands only under legacy section
- old v4/v3/v2 README sections removed
- external AI proof explained
- US and CN sample gallery present
- sample links exist
- no `research-rs/reports` generated
- generated paths anchor to repo root
- AI artifacts have provenance
- v5 sample reports use v5 structure

## 11. Final Status

Status: PASS

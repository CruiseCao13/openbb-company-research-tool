# v5 Version Consistency Scan

Generated: 2026-05-25 Asia/Singapore

## Scope

Scanned `README.md`, `docs`, `research-rs`, `providers`, `scripts`, `eval_sets`, `reports/release_checks`, `examples`, and `tests` for old v2/v3/v4 wording, legacy Python entry points, and template-driven workflow language.

Command pattern:

```bash
rg -n "v4\.|v3\.|v2\.|v4\.3|v4\.4|v4\.2|v3\.0|v2\.0|asset-aware workflow|openbb-research|cresearch|company_research_tool.py|old Python|legacy|template-driven" ...
```

## Classification

| Location | Classification | Action |
|---|---|---|
| `README.md` | Current v5 product page | PASS: only a short Legacy Python Workflow pointer remains. |
| `docs/history_v2_v4.md` | Accepted history | PASS: old v2-v4 narrative moved here. |
| `docs/cli_usage.md` | Legacy Python workflow | FIXED: rewritten as legacy-only and points to `research-rs` as current. |
| `docs/batch_evaluation.md` | Legacy Python workflow | FIXED: marked as pre-v5 Python batch workflow. |
| `docs/report_structure.md` | Legacy Python workflow | FIXED: marked as pre-v5 report structure. |
| `examples/example_commands.md` | Mixed examples | FIXED: v5 `research-rs` examples are first; legacy commands are explicitly marked. |
| `scripts/` | Legacy implementation | ACCEPTED: old Python workflow retained for compatibility, not the v5 control plane. |
| `tests/test_*v4*`, asset-aware tests | Legacy regression tests | ACCEPTED: tests preserve old Python behavior and are not product docs. |
| `CHANGELOG.md`, `docs/development_log.md`, roadmap docs | Historical notes | ACCEPTED: version references are historical. |
| `research-rs/target` | Build artifacts | IGNORED: not source or product documentation. |

## Result

README no longer presents v4.3/v4.4/v3/v2 as the current product. Remaining old-version references are either legacy documentation, historical notes, compatibility tests, or retained Python implementation files.

Status: PASS

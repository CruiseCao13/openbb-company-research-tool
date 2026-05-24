# Deprecation Policy

## v4 Python CLI

The v4 Python CLI remains available as a reference and fallback path during the v5 alpha period. It should not be deleted until v5 has passed staged broad_30 and mixed US/CN quality runs.

## v5 Rust CLI

`research-rs` is the intended primary entry point for v5 research runs:

- `research-rs doctor`
- `research-rs run TICKER`
- `research-rs batch EVAL_SET`
- `research-rs quality EVAL_SET`
- `research-rs lint RUN_FOLDER`
- `research-rs pack RUN_FOLDER`
- `research-rs samples`

## Report Format Migration

v5 reports add typed AI artifacts, parser/normalizer reports, reproducibility manifests, evidence maps, data inventories, product quality scores, and static dashboards. Older v4 report folders can remain readable, but v5 validators may mark them as legacy if `schema_version` is missing.

## Migration Guidance

To compare v4 and v5:

1. Run the same ticker with the old Python CLI.
2. Run `research-rs run TICKER --mode standard`.
3. Compare `metadata/research_blueprint.json`, `audit/validator_report.md`, and `dashboard.html`.

Do not merge v4 and v5 artifacts into the same run folder.

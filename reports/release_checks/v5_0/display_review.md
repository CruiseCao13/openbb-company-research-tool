# v5.0 Display Review

| Area | Status | Notes |
|---|---|---|
| Terminal UX | PASS | The Rust CLI prints staged progress, status, frame, AI confidence, report path, and pack path. |
| Report UX | PASS | Markdown reports include fixed sections, money-flow explanation, blueprint, data gaps, self-review, and locked-data appendix. |
| Folder UX | PASS | Run folders contain report, raw, metadata, ai, audit, self_review, data, charts, pack, README, and dashboard. |
| Batch UX | PASS | The batch run writes batch_summary.md, failures.md, warnings.md, profile_distribution.md, credit_usage_estimate.md, company_matrix.csv, and dashboard.html. |
| HTML Dashboard | PASS | Static HTML dashboards are generated for single-company and batch runs. |
| PDF Export | PASS | A basic dependency-free PDF export is generated next to Markdown reports. |
| Content Quality UX | PASS | Quality runs generate summary, matrix, spot-check, chart/table, money-flow, and training-case reports. |
| Language Quality | WARNING | The current local compact analyst is clear and bounded, but it is still less nuanced than the planned external AI analyst. |

No display category is marked FAIL.

## Chart / Table Review

- Charts generated: Yes, via `providers/chart_provider.py`.
- Chart numbering: PASS, files use `Figure_01...Figure_05`.
- Chart explanations: PASS, Markdown includes What to look at / What it means / What not to overread / Next check blocks.
- Table width: PASS, report tables are kept to four columns or fewer.
- Table units and source notes: PASS for status, money-flow, and locked-data tables.
- Raw NaN/null/placeholders: PASS in generated report surface.
- PDF export: PASS for AAPL and 600519.SH validation runs; the current exporter is text-first and does not embed PNG chart images.

## Dashboard Review

- Single-company dashboard exists.
- Batch dashboard exists.
- No external CDN or React build step is required.
- Dark-mode first CSS is inline.

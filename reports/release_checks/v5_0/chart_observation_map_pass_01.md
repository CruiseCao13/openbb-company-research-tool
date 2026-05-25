# v5 Chart Observation Map Pass 01

Date: 2026-05-25

Final status: PASS

This pass converts v5 English report chart explanations from generic figure-type boilerplate into company-specific chart observations. It does not call external OpenAI API, does not generate D3, does not change providers, does not run broad_500, and does not change dashboard/PDF styling.

## Files Changed

- `research-rs/crates/research-core/src/validation.rs`
- `research-rs/crates/research-report/src/markdown.rs`
- `research-rs/crates/research-report/src/renderer.rs`
- `research-rs/crates/research-report/src/tests.rs`
- `reports/release_checks/v5_0/chart_observation_map_pass_01.md`
- `reports/release_checks/v5_0/template_leakage_audit_03.md`

## New Chart Artifacts

Each rerun now generates:

- `metadata/chart_observation_map.json`
- `metadata/chart_claim_map.json`
- `audit/chart_observation_quality_report.md`

`chart_observation_map.json` contains, per chart:

- `chart_id`
- `title`
- `source_file`
- `source_fields`
- `company_specific_observation`
- `linked_financial_metric`
- `linked_company_fact`
- `what_this_chart_supports`
- `what_this_chart_cannot_prove`
- `next_check`
- `data_gap_if_any`
- `confidence`

`chart_claim_map.json` maps chart claims to:

- report section
- evidence refs
- overread risk
- limitation text

## Renderer / Validation Changes

- Chart blocks now display `Company-specific observation:` instead of generic `What to look at:` boilerplate.
- The observation includes ticker, source fields, latest locked observation, research frame, support boundary, limitation, and next check.
- Visual lint accepts `Company-specific observation:` as a valid chart explanation block.
- Chart limitation wording avoids forbidden advice phrases.

## Samples Rerun

Command pattern:

`cargo run --manifest-path research-rs/Cargo.toml -p research-rs -- run TICKER --market MARKET --provider auto --ai local --run-id chart_observation_map_01_TICKER_SAFE --force`

No external OpenAI API was used.

| Ticker | Run ID | Report status | Chart observation map | Chart claim map | Chart quality | Charts | Old generic chart phrase hits |
|---|---|---|---|---|---|---:|---:|
| AAPL | `chart_observation_map_01_aapl` | PASS | PASS | PASS | PASS | 5 | 0 |
| GOOGL | `chart_observation_map_01_googl` | PASS | PASS | PASS | PASS | 5 | 0 |
| CAT | `chart_observation_map_01_cat` | PASS | PASS | PASS | PASS | 5 | 0 |
| ISRG | `chart_observation_map_01_isrg` | PASS | PASS | PASS | PASS | 5 | 0 |
| JPM | `chart_observation_map_01_jpm` | PASS | PASS | PASS | PASS | 5 | 0 |
| RKLB | `chart_observation_map_01_rklb` | WARNING | PASS | PASS | PASS | 5 | 0 |
| 600519.SH | `chart_observation_map_01_600519_sh` | PASS | PASS | PASS | PASS | 5 | 0 |
| 000001.SZ | `chart_observation_map_01_000001_sz` | PASS | PASS | PASS | PASS | 5 | 0 |
| 300750.SZ | `chart_observation_map_01_300750_sz` | PASS | PASS | PASS | PASS | 5 | 0 |
| 601318.SH | `chart_observation_map_01_601318_sh` | PASS | PASS | PASS | PASS | 5 | 0 |
| 600276.SH | `chart_observation_map_01_600276_sh` | PASS | PASS | PASS | PASS | 5 | 0 |
| 601899.SH | `chart_observation_map_01_601899_sh` | PASS | PASS | PASS | PASS | 5 | 0 |

RKLB remains report-level WARNING because local fallback is appropriately data-limited for speculative aerospace. Chart observation quality is PASS.

## Repeated Chart Phrase Count

Before this pass, representative chart phrases repeated across the 12 local reports, including:

- `Compare the company price path with the benchmark`
- `This can show whether the stock has created relative price`
- `A price chart cannot prove the stock is cheap`
- `The useful question is whether growth is converting`
- `A cash-flow bridge cannot prove future runway`

After rerun:

- Old generic chart phrase hits: 0
- `audit/chart_observation_quality_report.md` status: PASS for 12/12
- `metadata/chart_observation_map.json` generated: 12/12
- `metadata/chart_claim_map.json` generated: 12/12

## Generic Chart Explanations Remaining

Blocking generic chart explanations remaining: none detected in the old phrase scan.

Remaining repeated report text is outside chart observations:

- report status explanation
- Money Flow artifact-policy text
- locked data coverage table guidance
- AI self-review table scaffolding

Those are tracked as non-chart template residue and should be handled separately if needed.

## Chart Source Issues

No chart source issue was found in the 12 reruns:

- every run had `charts/chart_manifest.json`
- every run had 5 chart observation rows
- chart observation quality was PASS for every run

## Chart Limitation Issues

No chart limitation issue was found in the 12 reruns:

- every chart observation includes `what_this_chart_cannot_prove`
- every chart claim includes `unsupported_or_overread_risk`
- chart text avoids buy/sell/target-price implications

## Validation Commands

Focused validation during implementation:

- `cargo test --manifest-path research-rs/Cargo.toml -p research-core -p research-report`
- `cargo test --manifest-path research-rs/Cargo.toml -p research-report`

Full validation is recorded in the final response.

## Final Assessment

The chart-specific blocker from Company-Specific Analysis Pass 01 is resolved. v5 still has some repeated non-chart scaffolding, but chart explanations now use company-specific observations and locked-data evidence.

Final status: PASS

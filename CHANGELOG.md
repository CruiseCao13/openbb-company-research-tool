# Changelog

## v4.3.0

### Added

- Asset-aware research routing through `asset_profile` and multi-metric thesis spine generation
- Profile-specific interpretation blocks for Mature Compounder, Speculative / Unprofitable Growth, Capital-Intensive Semiconductor Turnaround, Financials, Cyclicals, Hybrid Growth, and Unknown / Data-Limited Screening
- AI interpretation patch loop with `correction_patch.json`, `patched_report_blocks.json`, and `patch_diff_log.md`
- Company specificity and lifecycle logic checks to catch profile/body mismatch
- `lifecycle_logic_report.md` for lifecycle/profile failure attribution
- Fallback usage tracking and visible framework coverage warnings
- Organized report pack layout: `report/`, `charts/`, `data/`, `audit/`, `ai/`, `dashboard/`, `metadata/`, and `self_review/`
- Automatic run README, asset profile metadata, report status metadata, framework gap analysis, improvement suggestions, and regression test suggestions
- `openbb-research` console command alias
- `--pack` and `openbb-research pack RUN_FOLDER` report-pack zip support
- v4.3 asset-routing tests covering AAPL-like, RKLB-like, semiconductor-turnaround, biotech-like, REIT-like, insurance/financial, cyclical, unknown, patch, fallback, CLI help, and output-pack scenarios

### Changed

- Version bumped to `4.3.0`
- Default report language is now `both`; use `--en` or `--zh` for single-language output
- Data audit generation is enabled by default; use `--no-audit-data` to disable it
- English and Chinese reports now read from asset-aware patched report blocks instead of generic mature-company wording
- Chart and dashboard files are organized into product-style run folders
- Terminal banner now describes the asset-aware workflow

### Fixed

- Prevented mature-compounder template contamination in speculative-growth reports
- Prevented INTC-like semiconductor manufacturing turnarounds from being reduced to generic technology screening-only
- Prevented earnings/PE-style valuation framing from becoming the core logic for unprofitable growth companies
- Prevented lifecycle failures from lacking a Markdown attribution report
- Prevented AI correction logs from staying disconnected from the final report text
- Prevented unknown/data-limited companies from silently receiving confident template conclusions

## v4.2.0

### Added

- Report Experience pass for more readable English and Chinese research memos
- `--language {en,zh}` and `--term-style {pure,bilingual}`
- Chinese status card with localized gate labels and research profile labels
- Chinese metric display labels for core price, quality, valuation, and resilience tables
- Chart walkthrough blocks for every major chart
- Key Questions and Answers sections with answer, evidence, and boundary
- Language quality fields in `language_lint_report.md`: mixed-language hits, translationese hits, unexplained charts, unanswered questions, raw placeholders, and table explanation checks

### Changed

- Chinese reports now avoid English system labels by default
- Missing placeholders are mapped to reader-facing text in final reports
- English report structure now starts with a reader-focused status card, core view, AI red flags, battle card, key questions, chart walkthrough, and next steps

## v4.0.0

### Added

- Four-gate workflow: Data Audit Gate, Risk Method Gate, AI Analyst Review Gate, and Language Lint Gate
- Data audit outputs: `data_audit.md` and `data_audit.csv`
- AI correction log outputs: `ai_correction_log.md` and `ai_correction_log.json`
- Price label sanity check output: `price_label_sanity_check.md`
- Risk Metric Methodology section
- Research Battle Card with long bet, short trigger, market pricing, what must hold, kill criteria, and verification priority
- Chinese report mode via `--cn` / `--chinese`
- Language lint report: `language_lint_report.md`
- Valuation Sensitivity section
- Segment Revenue manual-required module
- `python -m openbb_company_research_tool` entrypoint

### Changed

- Version bumped to `4.0.0`
- Repositioned project as a thesis-driven first-pass equity research workflow generator
- Reframed balance-sheet risk as Balance Sheet Resilience Score with explicit score direction
- Added `--price-field` and `--annualization-days` to make risk methodology explicit
- Added `UNVERIFIED_` naming for failed report and gate-log outputs

## v3.0.0

### Added

- Optional AI Review layer using OpenAI Chat Completions structured outputs
- `scripts/ai_review.py` with Pydantic schema, compact payload builder, API call, Markdown renderer, and graceful fallback
- `scripts/terminal_ui.py` with Rich terminal output and plain-print fallback
- CLI flags: `--ai-review`, `--ai-model`, `--ai-review-depth`, `--ai-timeout`, `--ai-max-output-tokens`, and `--no-rich`
- `.env.example` with `OPENAI_API_KEY` and `OPENAI_MODEL`
- Tests for AI review payloads, skipped fallback, argument parsing, and terminal fallback

### Changed

- Version bumped to `3.0.0`
- Deterministic report data is aggregated into a structured dictionary before AI review and report rendering
- README and report structure docs now explain: Data calculates. AI reviews. Human decides.
- Dependencies now include `openai`, `pydantic`, and `rich`

## v2.1.0

### Added

- Beginner Summary table for business quality, growth, valuation, balance-sheet risk, stock risk, and data confidence
- How to Read This Report section near the top of generated reports
- Plain-English meaning notes under major report sections
- Chart-reading notes for actual close price, normalized performance, drawdown, growth quality, ruin risk, and score charts
- Beginner warning under Research Score
- `docs/metric_guide.md` with beginner-friendly definitions, why each metric matters, and common beginner mistakes

### Changed

- README now describes the beginner clarity layer, metric guide, and chart-reading notes
- Report structure documentation updated for the v2.1 reading flow
- Generated reports keep professional metrics while making the interpretation path easier for non-specialist readers

## v2.0.0

### Added

- Professional report surface and GitHub README
- Plotly interactive HTML dashboard
- Research score radar HTML
- Static score component chart
- Static growth and quality trend chart
- Static ruin-risk snapshot chart
- Ruin Risk with debt, EBITDA, debt/FCF, cash runway, and heuristic fragility score
- Sanity Checks with severity, finding, and action columns
- Sector / lifecycle-aware research profiles
- Category-aware score weights
- Archive-by-default output behavior with `latest` refreshed as the newest copy
- Optional personal margin stress table with `--account-equity` and `--margin-loan`

### Changed

- Reports are now framed as a structured company research workflow instead of a passive data pack
- `--archive` is now a compatibility flag because every run is archived by default
- README now emphasizes sample charts, risk boundaries, and clear usage
- Research Score is less one-size-fits-all and shows its profile

## v1.3.0

### Added

- `latest` and `runs` output structure
- `--archive` and `--run-id` CLI options
- Actual close price chart
- Automatic data warnings
- One-line Verdict section
- Report structure documentation
- Sample report and charts for GitHub display
- Minimal tests for formatting, output paths, and chart generation

### Changed

- README rewritten for GitHub-ready presentation
- Metric formatting now uses an explicit display registry
- ETF / fund-like instruments skip company financial statement analysis
- Valuation Snapshot is grouped by analytical category
- Empty warning state uses clearer human-readable language

## v1.2.0

### Added

- First-time setup script
- Cleaner output naming
- Key Takeaways
- Data Confidence section
- Drawdown chart
- Cleaner percent formatting
- Empty financial row cleanup
- Score explanation section

## v1.1.0

### Added

- Advanced benchmark comparison
- Risk-adjusted metrics
- Research Score
- Cross-ticker comparison

## v0.1.0

### Added

- Initial working prototype

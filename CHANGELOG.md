# Changelog

## v2.0.0

### Added

- Bilingual report surface and GitHub README
- Plotly interactive HTML dashboard
- Research score radar HTML
- Static score component chart
- Static growth and quality trend chart
- Static ruin-risk snapshot chart
- Ruin Risk Snapshot with debt, EBITDA, debt/FCF, cash runway, and heuristic fragility score
- Sanity Scan with severity, finding, and action columns
- Sector / lifecycle-aware research profiles
- Category-aware score weights
- Archive-by-default output behavior with `latest` refreshed as the newest copy
- Optional personal margin stress table with `--account-equity` and `--margin-loan`

### Changed

- Reports are now framed as a "research radar" instead of a passive data pack
- `--archive` is now a compatibility flag because every run is archived by default
- README now emphasizes sample charts, risk boundaries, and bilingual usage
- Research Potential Score is less one-size-fits-all and shows its profile

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
- Research Potential Score
- Cross-ticker comparison

## v0.1.0

### Added

- Initial working prototype

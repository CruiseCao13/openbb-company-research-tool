# CLI Usage

Primary command:

```bash
openbb-research AAPL
```

Compatibility command:

```bash
cresearch AAPL
```

Module command:

```bash
.venv/bin/python -m openbb_company_research_tool AAPL
```

## Common Examples

```bash
openbb-research RKLB --both --full
openbb-research AAPL --en
openbb-research AAPL --zh
openbb-research NVDA --benchmark QQQ
openbb-research RKLB --run-id stress_rklb_v43
openbb-research INTC --both --full --pack --run-id stress_intc_v43
openbb-research pack reports/INTC/runs/stress_intc_v43
```

## Defaults

v4.3 defaults:

- benchmark: `SPY`
- start: `2023-01-01`
- language: `both`
- data audit: enabled
- archive: enabled
- rich terminal UI: enabled

Use `--no-audit-data` to disable data audit output.

## Packing a Run

Use `--pack` during generation to create `TICKER_research_pack.zip` inside the run folder.

Use the `pack` subcommand when the run already exists:

```bash
openbb-research pack reports/TICKER/runs/RUN_ID
```

The zip includes:

- `README.md`
- `report/`
- `charts/`
- `data/`
- `audit/`
- `ai/`
- `dashboard/`
- `metadata/`
- `self_review/`

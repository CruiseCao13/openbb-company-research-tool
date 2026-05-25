# Legacy Python CLI Usage

This document describes the pre-v5 Python workflow. It is retained for compatibility and historical comparison only.

The current v5 product entry point is `research-rs`; see [README.md](../README.md) for the primary quick start.

## Legacy Commands

```bash
openbb-research AAPL
cresearch AAPL
.venv/bin/python -m openbb_company_research_tool AAPL
```

## Legacy Examples

```bash
openbb-research RKLB --both --full
openbb-research AAPL --en
openbb-research AAPL --zh
openbb-research NVDA --benchmark QQQ
openbb-research RKLB --run-id stress_rklb_v43
openbb-research INTC --both --full --pack --run-id stress_intc_v43
openbb-research pack reports/INTC/runs/stress_intc_v43
```

## Legacy Defaults

The v4.3 Python workflow used these defaults:

- benchmark: `SPY`
- start: `2023-01-01`
- language: `both`
- data audit: enabled
- archive: enabled
- rich terminal UI: enabled

## Legacy Packing

```bash
openbb-research pack reports/TICKER/runs/RUN_ID
```

For v5 packs, use `research-rs pack RUN_FOLDER`.

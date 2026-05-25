# Example Commands

## v5 Primary CLI

```bash
cd research-rs
cargo run -p research-rs -- --help
cargo run -p research-rs -- run AAPL --ai local --run-id demo_aapl_local
cargo run -p research-rs -- run 600519.SH --market cn --provider akshare --ai local --run-id demo_600519
cargo run -p research-rs -- batch ../eval_sets/broad_30_probe.yaml --mode batch --workers 2 --ai local --run-id demo_broad_30
```

## Legacy Python Workflow

These commands belong to the pre-v5 Python workflow and are retained only for compatibility:

```bash
cresearch AAPL
cresearch AAPL TSLA RKLB
cresearch TSLA --benchmark VOO
cresearch NVDA MSFT --benchmark QQQ
cresearch TSLA --benchmark AAPL --start 2020-01-01
cresearch AAPL --risk-free-rate 0.04
```

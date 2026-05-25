# v5 CLI Consistency Report

Generated: 2026-05-25 Asia/Singapore

## Scan

Searched `README.md`, `docs`, `examples`, `tests`, `scripts`, and `research-rs` for `openbb-research`, `cresearch`, `python scripts/company_research_tool.py`, `research-rs`, and `cargo run`.

## Findings

| Area | Result | Notes |
|---|---|---|
| README Quick Start | PASS | Primary entry is `research-rs`; old Python commands do not appear in Quick Start. |
| README legacy section | PASS | `openbb-research`, `cresearch`, and `python scripts/company_research_tool.py` appear only as legacy compatibility commands. |
| `docs/cli_usage.md` | PASS | Rewritten as Legacy Python CLI Usage. |
| `docs/batch_evaluation.md` | PASS | Marked as pre-v5 Python batch workflow. |
| `examples/example_commands.md` | PASS | v5 `research-rs` examples are first; legacy commands are explicitly separated. |
| `scripts/` | ACCEPTED | Old Python CLI remains as compatibility implementation. |
| Python tests | ACCEPTED | They intentionally test retained legacy behavior. |
| v5 Rust docs/code | PASS | `research-rs` is the Rust CLI package and current control plane. |

## Result

The main product surface now points users to `research-rs`. Legacy Python commands are no longer presented as the current workflow.

Status: PASS

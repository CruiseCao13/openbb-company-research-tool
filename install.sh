#!/usr/bin/env bash
set -euo pipefail

echo "== OpenBB Research Engine v5.0 installer =="

if ! command -v rustc >/dev/null 2>&1; then
  echo "Rust is required. Install from https://rustup.rs/ and rerun this script."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo is required. Install Rust with rustup and rerun this script."
  exit 1
fi

PYTHON_BIN="${PYTHON_BIN:-python3}"
if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "Python 3 is required for provider adapters."
  exit 1
fi

if [ ! -d ".venv" ]; then
  "$PYTHON_BIN" -m venv .venv
fi

. .venv/bin/activate
python -m pip install --upgrade pip setuptools wheel
python -m pip install -r requirements.txt

if [ -f "requirements-v5.txt" ]; then
  python -m pip install -r requirements-v5.txt
fi

cargo build --manifest-path research-rs/Cargo.toml --bin research-rs

echo ""
echo "Installed. Next commands:"
echo "  research-rs/target/debug/research-rs doctor"
echo "  research-rs/target/debug/research-rs run AAPL --mode standard --pack"
echo "  research-rs/target/debug/research-rs batch eval_sets/broad_30_probe.yaml --mode batch --limit 5"

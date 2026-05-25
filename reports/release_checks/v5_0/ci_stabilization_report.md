# CI Stabilization Report

Generated: 2026-05-25

## Local CI Reproduction

Commands run:

```bash
cargo fmt --manifest-path research-rs/Cargo.toml --all -- --check
cargo clippy --manifest-path research-rs/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path research-rs/Cargo.toml
python3 -m py_compile providers/*.py
cargo build --manifest-path research-rs/Cargo.toml --bin research-rs
research-rs/target/debug/research-rs doctor
```

## Result

Local CI result: PASS

## Fix

- Main CI provider syntax check now uses `python3`.
- Manual provider-smoke dependency install now uses `python3`.

## Notes

- Main CI does not require `OPENAI_API_KEY`.
- Main CI does not call external OpenAI API.
- Main CI does not require `.env`.
- Main CI does not require real provider network access.
- `research-rs doctor` exits 0 with optional providers/API key missing; those are warnings, not push-CI blockers.

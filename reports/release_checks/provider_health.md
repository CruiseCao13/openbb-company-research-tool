# Provider Health

| Check | Status |
|---|---|
| Rust | rustc 1.95.0 (59807616e 2026-04-14) |
| Cargo | cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| Python | Python 3.13.2 |
| Python venv | Python 3.13.2 |
| OpenBB provider installed | yes |
| AKShare installed | yes |
| Tushare installed | yes |
| Baostock installed | yes |
| TUSHARE_TOKEN set | no |
| Eastmoney public fallback available | yes |
| PDF engine | lightweight local exporter available |
| AI key | external AI key configured |
| Cache directory | exists |
| Write permission | current workspace writable |

## A-share Provider Health

- Missing AKShare/Tushare/Baostock packages are WARNING conditions for optional A-share depth, not default doctor failures.
- Missing TUSHARE_TOKEN is a WARNING unless a Tushare-only run is requested.
- Eastmoney public fallback is labeled as `eastmoney_public`, `package_used=false`, `mock=false`, and `provider_adapter=akshare_compatible_fallback`.
- Mock data must never be labeled as a real provider.

No API keys or secrets are printed in this report.

Next: See `docs/error_handbook.md` if a provider or PDF export step fails.

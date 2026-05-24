# Normalizer Report

Status: PASS

Provider data was normalized before report rendering. Non-finite numeric values and empty metric rows are removed from normalized outputs; raw provider data remains locked in `raw/provider_payload.json`.

| Output | Rows / Fields |
|---|---:|
| normalized income rows | 150 |
| normalized balance rows | 151 |
| normalized cash-flow rows | 147 |
| normalized price points | 260 |
| valuation raw keys | 7 |

Renderer and dashboard should use typed normalized data or typed AI artifacts, not ad hoc raw JSON field access.

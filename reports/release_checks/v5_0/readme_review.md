# README Review — v5.0

| Area | Status | Notes |
|---|---|---|
| Bilingual surface | PASS | README includes English and Chinese sections with natural product framing. |
| Product positioning | PASS | It explains that the tool is a first-pass research memo engine, not an AI stock picker. |
| Architecture diagram | PASS | Mermaid pipeline diagram is present, with text fallback responsibility map. |
| Folder structure | PASS | Run folder and expanded output pack examples are shown. |
| Sample outputs | PASS | Sample dashboard path and sample report folders are listed. |
| Install / onboarding | PASS | README points to `install.sh`, `.env.example`, `research.toml.example`, `research-rs doctor`, and sample gallery generation. |
| Quality evaluation | PASS | Content quality outputs and rubric purpose are described. |
| Mock/fallback boundary | PASS | README states external AI is not enabled and A-share normalization may fallback. |
| Limitations and disclaimer | PASS | It clearly says no buy/sell/target price and no replacement for due diligence. |

## Three-minute user test

A first-time reader can identify what the engine does, how to run one company, how to run batch/quality checks, where outputs live, and how to inspect quality boundaries within three minutes.

## Remaining README Risk

The README is product-grade for v5 alpha. It should be refreshed again when external AI and full A-share adapters are enabled so the fallback boundary remains accurate.

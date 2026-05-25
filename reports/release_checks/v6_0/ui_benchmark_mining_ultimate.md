# v6 Ultimate UI Rebuild Benchmark Mining

## Scope

This benchmark pass reviewed product and open-source UI references before the v6 Studio front-end rebuild. No v5 research logic, providers, prompts, validators, financial calculations, training code, or generated run folders were modified.

## Projects Inspected

| Source | Location inspected | License notes | Useful patterns |
|---|---|---|---|
| OpenBB | `/tmp/v6-ui-benchmarks/OpenBB` | AGPL-3.0. No code copied. | Finance workspace framing, artifact-first workflows, separation between data tools and visual workspace. |
| FinceptTerminal | `/tmp/v6-ui-benchmarks/FinceptTerminal` | AGPL/commercial terms with trade dress restrictions. No code copied. | Dense terminal posture, multi-tool finance workspace organization, status-heavy research flow. |
| shadcn-admin | `/tmp/v6-ui-benchmarks/shadcn-admin` | MIT. No code copied. | Sidebar organization, settings/command layout patterns, compact controls, dashboard density. |
| d3-sankey | `/tmp/v6-ui-benchmarks/d3-sankey` | BSD-style license. Existing project dependency/API used; no source copied. | Sankey node/link mental model, graph layout vocabulary. |

## Patterns Borrowed

- Search-first product entry instead of treating the run list as the homepage.
- Clear workspace separation: landing, research workspace, matrix workspace, settings center.
- Left rail as secondary navigation, not the main visual object.
- Large graph-first center stage for money-flow understanding.
- Primary action dock near the company header so report/dashboard/PDF/chart/audit actions are immediately visible.
- Right-side instrument board with gauge-style status summaries instead of plain metric cards.
- Collapsed diagnostics and advanced path handling to avoid a debug-panel first impression.
- Liquid-glass zones with soft radius hierarchy, translucent gradients, blur, and subtle shadows.

## Patterns Rejected

- Auth/user/SaaS template areas from admin dashboards.
- Heavy card-grid-within-card-grid layouts.
- Full template import or wholesale visual copying.
- Raw file browser metaphors as a primary surface.
- Fake demo data for loaded-run state.
- Quantitative Sankey widths without numeric locked money-flow values.

## Code Copying

No benchmark source code was copied into the repository. The rebuild adapts design patterns and keeps the existing local implementation and existing dependencies.

## Applied Design Decisions

- The landing state now centers an example vascular qualitative money-flow preview with minimal copy and a search/analyze console.
- The loaded-run research state keeps Money Flow as the first visual object, then action dock, chart strip, tabs, and secondary detail.
- The settings center remains in-session and uses the local i18n/settings state rather than adding a heavy UI framework.
- Browser preview is treated honestly: it can capture landing/settings/fallback states, but real run and matrix artifact loading require Tauri IPC.

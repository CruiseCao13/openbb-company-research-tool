# v6 Ultimate UI Rebuild Benchmark Mining

## Scope

This benchmark pass reviewed product and open-source UI references before the v6 Studio front-end rebuild. No v5 research logic, providers, prompts, validators, financial calculations, training code, or generated run folders were modified.

## Projects Inspected

| Source | Location inspected | License notes | Useful patterns |
|---|---|---|---|
| OpenBB | `/tmp/v6-ui-benchmarks/OpenBB` | AGPL-3.0. No code copied. | Finance workspace framing, artifact-first workflows, separation between data tools and visual workspace. |
| FinceptTerminal | `/tmp/v6-ui-benchmarks/FinceptTerminal` | AGPL/commercial terms with trade dress restrictions. No code copied. | Dense terminal posture, multi-tool finance workspace organization, status-heavy research flow. |
| shadcn-admin | `/tmp/v6-ui-benchmarks/shadcn-admin` | MIT. No code copied. | Sidebar organization, settings/command layout patterns, compact controls, dashboard density. |
| Eigent AI | `/tmp/v6-ui-benchmarks/eigent` | Apache-2.0. No code copied. | AI workspace separation, agent/activity surfaces, side-panel depth, progressive disclosure of long details. |
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
- Micro-inspector tooltip behavior instead of browser default title tooltips.
- Dark crystal skeleton loading to prevent white/black hard flashes during run switching.
- Subtle scrollbar styling so scroll zones do not look like platform-default admin panes.
- Desaturated technical badges with small status orbs so warnings/data gaps get attention without traffic-light blocks.
- Settings/modal depth model: blurred overlay, high-radius glass sheet, smooth transform/opacity transitions.
- Keyboard-first command palette pattern from terminal/admin workspaces: Cmd/Ctrl+K exposes run loading, matrix, settings, and artifact commands without turning the app into a generic command-center product.
- Split-pane resize pattern from professional desktop tools: invisible hover gutters reveal a thin light line and persist rail widths instead of using heavy grey dividers.
- Optional tactile feedback pattern from hardware-style consoles: very low-volume Web Audio click on deliberate high-value actions, user-controlled in settings and gesture-only.
- Custom text selection and focus glow patterns so browser defaults do not break the black-gold visual system.

## Patterns Rejected

- Auth/user/SaaS template areas from admin dashboards.
- Heavy card-grid-within-card-grid layouts.
- Full template import or wholesale visual copying.
- Raw file browser metaphors as a primary surface.
- Fake demo data for loaded-run state.
- Quantitative Sankey widths without numeric locked money-flow values.
- Loud traffic-light color blocks and hard neon fills.
- Default browser tooltips, thick scrollbars, and abrupt blank loading states.
- Whole-template imports, auth/user-management shells, or SaaS administration pages.
- Loud or always-on sound effects, hover sounds, startup sounds, or notification-like beeps.
- Heavy draggable split-pane libraries; a local pointer-event implementation is enough for the two-resizer layout.

## Code Copying

No benchmark source code was copied into the repository. The rebuild adapts design patterns and keeps the existing local implementation and existing dependencies. Existing project dependencies such as `d3-sankey`, `i18next`, and `react-i18next` are used; no benchmark template was vendored.

## Applied Design Decisions

- The landing state now centers an example vascular qualitative money-flow preview with minimal copy and a search/analyze console.
- The loaded-run research state keeps Money Flow as the first visual object, then action dock, chart strip, tabs, and secondary detail.
- The settings center remains in-session and uses the local i18n/settings state rather than adding a heavy UI framework.
- Browser preview is treated honestly: it can capture landing/settings/fallback states, but real run and matrix artifact loading require Tauri IPC.
- Loading states use dark liquid skeleton surfaces so switching runs does not collapse the layout.
- Hover details use micro-glass inspectors and `data-tooltip` surfaces instead of native browser tooltips.
- The matrix and research workspaces keep separate IA, with matrix staying a quality board rather than a run-detail widget.
- The command menu is intentionally minimal and keyboard-first: it operates existing IPC-backed actions and does not invent new run execution.
- Pane resizing uses CSS variables persisted in localStorage and keeps the visual divider nearly invisible until hover/drag.
- Micro-click feedback is optional, off by default, low volume, generated locally, and fails silently if the runtime blocks audio.
- SVG flow rendering follows D3/SVG polish patterns: geometric precision, smoother multi-stop gradients, and no hard color banding.
- The right rail now follows progressive disclosure patterns from AI workspaces: one headline signal plus foldable warnings, data gaps, framework checks, and blueprint details.
- Metric rows use baseline-aligned value/unit styling to avoid optically loose financial numerals.
- Sub-1200px handling keeps the main graph prioritized by collapsing the insight rail into a hover/focus side overlay.

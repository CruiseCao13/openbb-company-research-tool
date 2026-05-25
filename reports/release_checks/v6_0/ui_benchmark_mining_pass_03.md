# v6 UI Benchmark Mining Pass 03

## Sources Reviewed

1. OpenBB Workspace / Platform
   - Source: https://www.openbb.co/products/platform
   - Useful pattern: finance workspace positioning between BI dashboard and terminal, with visualizations and AI analysis as first-class workspace objects.
   - Applied: kept Matrix and Research as separate workspaces; made charts and Money Flow the main objects rather than report-file browsing.

2. OpenBB new Workspace announcement
   - Source: https://openbb.co/blog/introducing-the-new-openbb-terminal
   - Useful pattern: shareable dashboard/report layers are supporting layers after visual exploration.
   - Applied: Report/PDF/Dashboard are obvious actions, but secondary to the visual money-flow workspace.

3. FinceptTerminal
   - Source: https://fincept.in/
   - Useful pattern: professional terminal framing, dense finance instrumentation, multi-asset analytics, and institutional charting.
   - Applied: Matrix was kept as a quality-board workspace; right rail became gauges instead of plain text cards.

4. FinceptTerminal GitHub
   - Source: https://github.com/Fincept-Corporation/FinceptTerminal
   - License note: AGPL/commercial license. No code copied.
   - Useful pattern: terminal-grade research breadth, native-app seriousness, and finance-specific instrumentation.
   - Applied: adapted only product principles; no source code was copied.

5. shadcn Command pattern
   - Source: https://www.shadcn.io/ui/command
   - Useful pattern: search should be an action surface, not decoration.
   - Applied: landing search console now controls ticker intent, market, mode, Analyze, Load Latest, and Matrix.

6. d3-sankey
   - Source: https://github.com/d3/d3-sankey
   - License note: open-source D3 ecosystem; existing dependency already used. No external code copied beyond normal package API usage.
   - Useful pattern: use Sankey for node/link layout, then customize SVG rendering.
   - Applied: kept `d3-sankey` for layout while rendering custom glowing vascular paths, filters, gradients, and hover-revealed labels.

7. Apple Human Interface Guidelines: Materials
   - Source: https://developer.apple.com/design/human-interface-guidelines/materials
   - Useful pattern: translucent materials create depth, hierarchy, and a sense of place without obscuring content.
   - Applied: expanded Liquid Glass classes and surfaces with translucent gradients, blur, inner highlights, and restrained shadows.

8. Eigent AI / Mission-control style agent workspaces
   - Sources:
     - https://www.eigent.ai/about
     - https://mc.builderz.dev/
   - Useful pattern: agent/workflow tools feel better when the workspace presents state and actions visually rather than as raw logs.
   - Applied: debug/log-like content was moved behind diagnostics/details; landing emphasizes visual state and controlled action.

9. OpenAI-style spatial product pages
   - Source: https://openai.com/
   - Useful pattern: large central object, few words, strong focus, and restrained visual hierarchy.
   - Applied: landing moved toward minimal copy plus a strong visual flow object and a focused search console.

## Code Copied

No external source code was copied. The pass adapted design patterns and used the existing `d3-sankey` dependency through its public API.

## Patterns Applied

- Graph-first research workspace.
- Search-first landing console.
- Separate Landing, Research, Matrix, and Settings modes.
- Large center visual object with text hidden until hover.
- Gauge-style status indicators instead of metric cards.
- Matrix as quality board rather than weak grid.
- Semantic artifact actions instead of file-path display.
- Liquid Glass zones and surfaces instead of heavy admin cards.

## Patterns Rejected

- Generic admin dashboard sidebars with CRUD-heavy pages.
- SaaS/auth/user-management template sections.
- Trading terminal live market feeds, because this pass must not call providers or external APIs.
- Quantitative Sankey without numeric DTO data, because that would fake precision.
- Direct copying from AGPL/commercial projects.

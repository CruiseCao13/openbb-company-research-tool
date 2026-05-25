# v6 UI Productization Pass 02 - Graph-First Liquid Glass Research Terminal

Date: 2026-05-26
Branch: v5-rust-ai-blueprint
Final status: WARNING

## Problems Addressed

- Continued moving the Studio away from a script/debug-panel feel and toward a graph-first desktop research terminal.
- Added stronger company identity treatment through local monogram tiles, without fetching external logos.
- Made settings more complete and session-persistent through localStorage.
- Hid full local paths from primary artifact actions and action result messages.
- Added pointer-following ambient glow, stronger Liquid Glass layering, rounded nesting, and Sankey flow animation.
- Added Matrix filter chips and stronger selected/hover affordances.

## Open-Source Inspirations Used

- shadcn-admin: adapted the organization pattern of navigation rail, settings surface, and dense dashboard controls. No code copied.
- Linear / Cursor style apps: adapted compact dark command/topbar and muted engineering-terminal hierarchy. No code copied.
- Apple/macOS spatial UI: adapted rounded nested surfaces, translucent depth, blur, soft highlights, and reduced-motion behavior. No code copied.
- D3 Sankey examples: used `d3-sankey` layout API for the flow visualization. No project-template code copied.

## Dependencies Added

No new dependencies were added in this pass. It reused dependencies already present from the previous UI pass:

- i18next
- react-i18next
- d3-sankey
- @types/d3-sankey

## Files Changed

- studio/src/App.tsx
- studio/src/components/RunDetailPanel.tsx
- studio/src/styles.css
- reports/release_checks/v6_0/screenshots/ui_pass_02_landing.png
- reports/release_checks/v6_0/ui_productization_pass_02_report.md

## New Information Architecture

- Landing / Opening Layer remains the first screen when no run is actively inspected.
- Research Workspace remains graph-first; default run detail tab is Charts, which starts with Money Flow Sankey.
- Matrix Workspace remains isolated from run detail language.
- Settings Center is accessible from the top chrome and from the left rail.

## Landing Hero Behavior

- Hero keeps the required English title and subtitle.
- Added a third CTA: View Quality Matrix.
- Added pointer-following shell glow and preserved slow industrial orbit motion.
- Landing hides raw paths and debug details.

## Language / Settings Behavior

- Settings Center now includes:
  - Language
  - Density
  - Motion
  - Glass intensity
  - Font scale
  - Show warnings first
  - Default landing view
  - Start on latest run
  - Open matrix by default
- Settings are persisted to localStorage for the session/browser profile.
- Language switch remains UI-chrome focused; artifact content is not translated.

## Chart-First Layout

- Default run tab remains Charts, not Summary.
- Charts tab starts with the D3 Money Flow Sankey, then existing chart artifact gallery.
- Missing chart artifacts keep honest empty states.

## D3 Sankey Status

- Implemented as a qualitative flow map using current RunDetail money-flow fields, blueprint data gaps, and provider missing fields.
- Link hover shows native SVG title text.
- Added animated flow shimmer and reduced-motion support.

## Qualitative vs Quantitative Sankey Honesty

- Current Sankey is explicitly labeled "Qualitative flow map" and "Not amount-scaled".
- No revenue, capex, FCF, dividend, debt, or buyback values are fabricated.
- Quantitative Sankey remains blocked until the Studio DTO exposes numeric locked money-flow fields.

## Company Logo / Monogram Behavior

- No external logo network calls.
- No broken image state.
- Run list and run header use a local monogram tile derived from ticker and market.
- Status colors influence monogram glow.

## Primary Action Visibility

- Report, Dashboard, PDF, Reveal Folder, AI Usage, Validator Audit, and Provider Payload remain in the top artifact dock.
- Action labels are product names, not local paths.
- Disabled actions explain missing artifacts via state/title.

## Path Hygiene Behavior

- Full local paths are not shown in primary action labels or action success messages.
- Reveal/open actions still use Tauri IPC and backend path validation.
- `run_id` remains visible as run identity; full filesystem paths stay out of the main action flow.

## Liquid Glass Implementation Details

- Added stronger rounded hierarchy:
  - outer shell/panels around 18-28px
  - monogram tiles around 16-20px
  - buttons/badges rounded to pill or soft capsule shapes
- Added translucent gradient surfaces, blur, inner highlights, soft shadows, selected glow, and layered depth.
- Added global pointer-following radial glow through CSS variables.

## Interaction Energy Implementation Details

- Pointer-following ambient glow on the shell.
- WARNING / FAIL / DATA_GAP / HUMAN_REVIEW / LOCAL_MOCK / PROVIDER_MOCK retain breathing badge animation.
- Sankey links now have subtle animated flow shimmer.
- Matrix cells have filter chips, hover lift, and selected glowing ring.
- Settings and diagnostics continue to use glass surfaces and reduced-motion safeguards.

## Diagnostics Drawer Behavior

- Diagnostics stays collapsed by default.
- Strip shows AI source, provider, warning count, data gap count, and human review signal.
- Expanded drawer keeps provenance/gaps secondary, not blocking the artifact dock or Sankey.

## Regression Matrix Improvements

- Added filter chips:
  - ALL
  - PASS
  - WARNING
  - FAIL
  - DATA_GAP
  - LOCAL
  - EXTERNAL
- Summary total now shows filtered/total rows.
- Existing legend and inspector remain visible.

## Motion / Cursor Glow Behavior

- Pointer glow is CSS-variable driven from React pointer events.
- Reduced motion disables transition/animation intensity.
- Motion toggle disables animation through `data-motion="off"`.

## Screenshots

Generated:

- reports/release_checks/v6_0/screenshots/ui_pass_02_landing.png

Blocked / not generated:

- loaded run overview
- charts with real IPC-loaded run data
- matrix with real training artifact data
- settings modal screenshot
- diagnostics drawer screenshot

Reason: automated screenshot capture used browser preview, where Tauri IPC is intentionally unavailable. Real loaded-state screenshots require desktop runtime click-through.

## Validation Results

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS
- `git diff --check`: PASS
- real secret scan: PASS

No external OpenAI API, provider network, training run, broad eval run, or v5 research pipeline command was invoked.

## Remaining UI Limitations

- Final status is WARNING because real desktop loaded-state screenshot/runtime review is still pending.
- Sankey remains qualitative because current Studio DTO does not expose numeric money-flow fields.
- Matrix filters are basic client-side filters, not a full query/filter system.
- Bilingual coverage is broad UI chrome coverage, but not every artifact-sourced text string.
- Tauri window transparency/titlebar customization was not changed in this pass to avoid startup risk.

# v6 UI Productization Pass 01 - Research Workspace

Date: 2026-05-26
Branch: v5-rust-ai-blueprint
Final status: PASS with remaining UI limitations

## Scope

This pass only changed the v6 Tauri Studio frontend and dependency layer. It did not modify v5 research logic, providers, validators, training, financial calculations, report generation, or run artifacts.

## Files Changed

- package.json
- package-lock.json
- studio/src/App.tsx
- studio/src/components/RunDetailPanel.tsx
- studio/src/i18n.ts
- studio/src/main.tsx
- studio/src/styles.css
- reports/release_checks/v6_0/screenshots/ui_productization_hero_landing.png
- reports/release_checks/v6_0/ui_productization_pass_01_report.md

## UI Changes

- Added a hero landing view with:
  - "See with Your Own Eyes"
  - "Trace the business. Verify the numbers. Follow the cash."
  - Enter Research Studio CTA
  - Open Latest Run CTA
  - subtle industrial orbit motion
- Added a global top bar with:
  - language switch
  - settings button
  - density switch
  - current mode readout
  - quick search
- Added a right-side insight rail for:
  - company identity
  - money flow
  - warnings
  - blueprint / next checks
- Added Settings Center with:
  - language
  - density
  - motion
  - glass intensity
  - font scale
  - show warnings first
  - default landing view
- Added a first bilingual UI chrome layer using react-i18next/i18next.
- Upgraded glass styling with larger border radii, softer shadows, blur, rounded controls, and more deliberate card hierarchy.

## Chart / Graph Changes

- Added `d3-sankey` and a Money Flow Sankey component in the Charts tab.
- Sankey is explicitly qualitative and read-only:
  - consumes existing RunDetail money-flow fields, blueprint data gaps, and missing fields
  - does not fabricate financial amounts
  - link width is not presented as amount-scaled
  - includes source, limitation, and next-check text
- Existing chart artifacts remain the source for gallery previews.
- Missing chart artifacts still show a data-gap state rather than broken images.

## Placeholders / Still Incomplete

- The Sankey is qualitative until v5 emits numeric money-flow map values suitable for amount-scaled visualization.
- Bilingual coverage is first-pass UI chrome only; detailed report/audit text remains artifact-provided.
- Settings are local component state, not persisted yet.
- Tauri window customization was not changed in this pass.
- Real loaded-run screenshots were not captured automatically; browser preview screenshot shows honest desktop-required IPC state.
- `npm install` reports 2 moderate npm audit findings in the dependency tree; no fix was applied because forced upgrades could destabilize the Studio stack.

## Validation Results

- `npm run typecheck`: PASS
- `npm run build`: PASS
- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo build --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS
- `git diff --check`: PASS

No external OpenAI API, provider network, training run, or v5 report pipeline command was invoked.

## Screenshot

- reports/release_checks/v6_0/screenshots/ui_productization_hero_landing.png

## Remaining UI Blockers

- Needs manual desktop runtime review with real loaded run data.
- Need a second pass for persisted settings and broader bilingual coverage.
- Need real run screenshot review for chart gallery, Sankey with loaded metadata, Matrix mode, and diagnostics drawer.

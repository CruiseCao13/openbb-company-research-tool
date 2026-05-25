# v6 Phase 01 Studio Shell Report

Date: 2026-05-25

## What Was Created

Phase 01 created the smallest runnable v6 Tauri Research Studio shell.

Created:

- `studio/` Vite + React + TypeScript frontend
- `src-tauri/` Tauri v2 backend
- root `package.json`
- root `package-lock.json`
- minimal `ping_studio()` Tauri command
- static dark industrial app shell UI

No v5 provider, AI prompt, validator, training, report generation, or financial calculation logic was modified.

## Directory Layout

```text
studio/
  index.html
  tsconfig.json
  vite.config.ts
  src/
    App.tsx
    main.tsx
    styles.css

src-tauri/
  Cargo.toml
  Cargo.lock
  build.rs
  tauri.conf.json
  icons/icon.png
  src/
    lib.rs
    main.rs
```

Tauri generated schema files under `src-tauri/gen/schemas/` during Rust validation.

## Frontend Stack

- Vite
- React
- TypeScript
- Plain CSS design tokens
- npm package manager

Tailwind CSS was intentionally not added in Phase 01 to keep the shell stable and minimal.

## Tauri Backend Status

Tauri backend status: PASS.

Implemented command:

```text
ping_studio()
```

Return shape:

```json
{
  "status": "ok",
  "message": "v6 studio shell ready"
}
```

No filesystem browsing command is implemented yet.
No provider call is implemented.
No OpenAI call is implemented.
No report parsing is implemented.

## Scripts Added

```json
{
  "dev": "vite --config studio/vite.config.ts --host 127.0.0.1",
  "build": "tsc -p studio/tsconfig.json && vite build --config studio/vite.config.ts",
  "typecheck": "tsc -p studio/tsconfig.json --noEmit",
  "tauri": "tauri",
  "tauri:dev": "tauri dev",
  "tauri:build": "tauri build"
}
```

## Validation Commands Run

Frontend:

```bash
npm install
npm run typecheck
npm run build
```

Tauri:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

Existing v5 Rust:

```bash
cargo test --manifest-path research-rs/Cargo.toml
```

## Validation Results

| Check | Result |
|---|---|
| npm install | PASS |
| npm run typecheck | PASS |
| npm run build | PASS |
| Tauri cargo check | PASS |
| Tauri cargo build | PASS |
| Existing v5 cargo test | PASS, 152 tests |

`npm install` reported 2 moderate npm audit advisories in JavaScript tooling dependencies. No breaking `npm audit fix --force` was applied in this scaffold pass.

## Intentionally Not Implemented

- Real run folder discovery
- Real filesystem commands
- Report parsing
- Dashboard parsing
- PDF viewing/export logic
- D3 charts
- Regression Matrix
- External OpenAI calls
- Provider calls
- Training or eval-set integration
- Any mutation of v5 run folders

## Next Phase Recommendation

Phase 02 should add read-only Tauri DTO commands behind strict path boundaries:

- `list_runs`
- `load_run_summary`
- `load_ai_usage`
- `load_provider_status`
- `load_artifact_availability`

React should continue to render DTOs only and should not read the filesystem directly.

## Final Status

PASS.

v6 Phase 01 produced only a static Tauri/React shell plus a minimal IPC command. It does not claim dashboard, run discovery, data loading, or report browsing is complete.

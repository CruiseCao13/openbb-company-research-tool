# Generated Artifact Hygiene Report

Generated: 2026-05-25 Asia/Singapore

## Evidence

Tracked generated/cache/temp matches:

```text
(none)
```

Relevant `.gitignore` entries:

```text
2:.venv/
3:__pycache__/
9:.DS_Store
14:.pytest_cache/
16:research-rs/target/
17:research-rs/training_cases/
21:research-rs/reports/
38:.env
39:.env.*
40:!.env.example
```

## Findings

| Check | Status | Evidence |
|---|---|---|
| `research-rs/reports/` absent | PASS | No cwd-anchored reports directory. |
| `reports/_cache/` not tracked | PASS | git tracked-file scan. |
| `.env` not tracked | PASS | git tracked-file scan. |
| Release checks can be committed | PASS | `reports/release_checks/**` is intentionally unignored. |
| Samples can be committed | PASS | `reports/samples/**` is intentionally unignored as product gallery. |
| Random run outputs ignored | PASS | `reports/*` ignores non-sample/non-release run folders. |
| `research-rs/training_cases/` absent | PASS | A stale cwd-rooted training case directory was removed after routing v5 correction cases to repo-root `training_cases/`. |

Status: PASS

---
type: is
id: is-01m3637s9hez8e47f6tw7svrhp
title: Install pinned uv in release rehearsal plan job
kind: bug
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex-alpha-surface-composition
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-23T02:59:08.208Z
updated_at: 2026-09-23T03:00:01.354Z
started_at: 2026-09-23T02:59:20.785Z
---
The Release rehearsal plan job runs Python release unit tests that invoke Flowmark through uv, but the job does not install uv. Rehearsal run 35812226235 fails with FileNotFoundError before artifact jobs can start. Add the existing SHA-pinned astral-sh/setup-uv action at the repository pin and version 0.12.1 before release unit tests. Keep the tests, verify workflow pin policy and rerun rehearsal.

## Notes

Added the repository’s SHA-pinned setup-uv v7 action and reviewed uv 0.12.1 before release plan tests. Local supply-chain/uv pin tests: 32 passed; release unit tests: 44 passed. Parent review and a new exact-head release rehearsal are pending.

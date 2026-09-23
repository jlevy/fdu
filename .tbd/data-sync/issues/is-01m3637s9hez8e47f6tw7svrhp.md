---
type: is
id: is-01m3637s9hez8e47f6tw7svrhp
title: Install pinned uv in release rehearsal plan job
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex-alpha-surface-composition
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-23T02:59:08.208Z
updated_at: 2026-09-23T08:14:06.911Z
started_at: 2026-09-23T02:59:20.785Z
closed_at: 2026-09-23T08:14:06.911Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
The Release rehearsal plan job runs Python release unit tests that invoke Flowmark through uv, but the job does not install uv. Rehearsal run 35812226235 fails with FileNotFoundError before artifact jobs can start. Add the existing SHA-pinned astral-sh/setup-uv action at the repository pin and version 0.12.1 before release unit tests. Keep the tests, verify workflow pin policy and rerun rehearsal.

## Notes

Added the repository’s SHA-pinned setup-uv v7 action and reviewed uv 0.12.1 before release plan tests. Local supply-chain/uv pin tests: 32 passed; release unit tests: 44 passed. Parent review and a new exact-head release rehearsal are pending.

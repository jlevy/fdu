---
type: is
id: is-01m3rg10nk2njtvckwk6a6n2dj
title: "Atomic writes: release tooling (scripts/release)"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:57.651Z
updated_at: 2026-09-30T06:46:38.630Z
closed_at: 2026-09-30T06:46:38.630Z
close_reason: "scripts/release writes go through scripts/atomic_write.py (maintainer.py state.json, download marker, notes-source.md, notes.md, notes.html, allowed_signers, registry-state.json; announce.py registry-state.json; inspect_artifacts.py manifest and checksums; registry_state.py --output; release_body.py source, body, flowmark input). The two $GITHUB_OUTPUT appends (publish_gate.write_outputs, resolve_plan) are listed exceptions in scripts/check-atomic-writes.mjs: runner-owned, a torn write fails its step, no release.yml step runs after a failure. make release-test: 214 tests OK; ruff format/check clean; each script runs by path. Commit c8e865a7 on claude/atomic-writes."
resolution: null
duplicate_of: null
---
About 16 plain writes in scripts/release (maintainer.py, release_body.py, inspect_artifacts.py, resolve_plan.py, registry_state.py, publish_gate.py, announce.py) go through an atomic helper.

---
type: is
id: is-01m3mfpqhjhwv9redfnv8e87ya
title: Run release.yml's release scripts on the pinned uv Python, not the runner's python3
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwy5q1j1nab2srxz38ebb
created_at: 2026-09-28T17:06:22.896Z
updated_at: 2026-09-28T17:06:22.896Z
---
release.yml installs setup-uv 0.12.1 but runs every release script with the runner's system python3: resolve_plan.py and the release unittest suite in the plan job (lines 71, 78), smoke_crate.py (104), inspect_artifacts.py and registry_state.py in the evidence job (240, 247), and resolve_plan.py, publish_gate.py, registry_state.py in the publish job (278-474). make release-test and make release-rehearse use uv run --no-project --python 3.12 instead. tbd guidelines release-engineering-rules: 'Give release-support code an interpreter or toolchain the project pins, not the host's' - an ubuntu-latest image update can change python3 at the least recoverable moment. Fix: run them through uv run --no-project --python 3.12 (setup-uv is already present in plan/sdist/wheels/publish; add it to crate/evidence/release-environment). Needs review of test_metadata.py guards, which pin parts of the workflow text. Found while streamlining the release process (fdu-n2hc).

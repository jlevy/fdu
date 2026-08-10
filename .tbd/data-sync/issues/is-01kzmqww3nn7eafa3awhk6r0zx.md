---
type: is
id: is-01kzmqww3nn7eafa3awhk6r0zx
title: Isolate the non-Unicode wheel smoke fixture
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - python
  - ci
dependencies:
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T02:26:45.493Z
updated_at: 2026-08-10T02:27:25.417Z
---
PR #2 Ubuntu wheel CI exposed that the non-Unicode argv fixture is created beneath the already indexed Python API fixture. Linux accepts the byte name, so the later refresh observes two unrelated insertions; APFS rejects the byte name and masked the coupling locally. Create the native-argv fixture in a separate temporary tree, retain the lossless argv assertions, rerun wheel smoke locally, and verify the expanded CI matrix.

## Notes

Root cause confirmed from PR #2 Ubuntu job 93337907802: Linux accepted the raw byte path inside the already indexed root, so refresh correctly reported three insertions instead of one. The argv fixture now uses a separate temporary parent. Local installed-wheel and uvx smoke pass; awaiting the refreshed Linux and Windows CI matrix.

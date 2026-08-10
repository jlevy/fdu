---
type: is
id: is-01kzmrp7p8sv40v43bw3y4t5wg
title: Compare canonical Python roots by filesystem identity
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - python
  - windows
dependencies:
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T02:40:36.551Z
updated_at: 2026-08-10T02:48:05.017Z
closed_at: 2026-08-10T02:48:05.015Z
close_reason: Fixed in 97ae2ba by comparing canonical roots with os.path.samefile while preserving Rust native verbatim path behavior. Final PR CI run 31350476952 passed the Windows installed-wheel smoke and direct uvx lane.
---
The expanded Windows wheel smoke showed that Rust canonicalize returns the native verbatim path spelling while Python os.path.realpath returns the conventional spelling. Both identify the same directory, so string equality is a platform-specific test bug and stripping the verbatim prefix could break long-path behavior. Compare with os.path.samefile, retain exact root-path behavior in the API, and verify the full Windows wheel and uvx lane.

## Notes

Windows wheel job 93339351300 showed that Rust canonical verbatim paths and Python conventional realpath spellings identify the same directory but are not string-equal. The smoke now compares filesystem identity with os.path.samefile and preserves native long-path behavior. Local installed-wheel and uvx smoke pass; awaiting refreshed Windows wheel CI.

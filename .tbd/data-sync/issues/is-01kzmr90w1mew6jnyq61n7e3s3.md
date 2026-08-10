---
type: is
id: is-01kzmr90w1mew6jnyq61n7e3s3
title: Normalize embedded skill line endings across platforms
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
  - windows
dependencies:
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T02:33:23.584Z
updated_at: 2026-08-10T02:34:21.075Z
---
PR #2 Windows native CI showed that Git checkout can convert the included SKILL.md to CRLF. include_str then made fdu --skill platform-dependent and failed the portable-skill assertion. Normalize CRLF to LF when composing the embedded skill, retain exact-version replacement and LF assertions, and verify native and wheel CI on Windows.

## Notes

Windows native job 93338506681 confirmed that include_str inherited CRLF from checkout and made fdu --skill platform-dependent. The composer now normalizes CRLF to LF before exact-version substitution. A deterministic CRLF-input assertion and all 26 golden scenarios pass locally; awaiting refreshed Windows CI.

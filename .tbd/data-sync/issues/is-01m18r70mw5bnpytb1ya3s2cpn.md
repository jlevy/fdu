---
type: is
id: is-01m18r70mw5bnpytb1ya3s2cpn
title: fdu rejects a regular file as a scan root
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - cli
dependencies: []
created_at: 2026-08-30T07:12:48.283Z
updated_at: 2026-08-30T07:12:48.283Z
---
'fdu README.md --view summary' fails with 'I/O error at <path>: scan root is not a directory', exit 1. Same on main and on codex/opened-root-inventory-rewrite, so this is not branch-specific.

Origin: a field agent reported that '--view summary' on a regular file 'produced no size row'. That specific observation does not reproduce - their path did not exist, so they saw a not-found error. The underlying question is still fair: du accepts a file argument and reports its size, fdu does not accept one at all.

Decide deliberately rather than by omission: either accept a file root and emit a one-entry roll-up, or keep rejecting it and make the message say roots must be directories. Either is defensible; silence about which is intended is the actual gap for scripting consumers.

Acceptance: behaviour for a regular-file root is a stated decision, covered by a test and by --help.

---
type: is
id: is-01m18r6049rg359vn5nr1tazky
title: Control-table bound is not liftable by any flag and its error names no remedy
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - control-state
  - cli
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.984Z
updated_at: 2026-08-30T07:12:14.984Z
---
MAX_CONTROL_TABLE_BYTES is a hard const with no CLI or config lever (verified: no match for control-table/max-control in crates/fdu/src). The error text is 'control table requires N bytes; limit is M bytes' - it states the bound and offers no way past it.

fdu-design-principles.md: 'Every bound is liftable by a flag named where the bound is stated. A truncation the caller cannot remove is a limitation wearing a default's clothes.'

A field agent independently hit exactly this and reported 'there is no flag to raise it'. --scan-depth is not a workaround: it limits scanning as well as retention, so roll-ups undercount and the answer changes. --min-size does not help either: selection applies to reporting, after the table is already built.

Fix direction: separate the two jobs the constant currently serves - keep a strict parser guard for snapshot loading (untrusted u32 lengths in snapshot.rs:688,722 must stay bounded), and add a separate, larger, flag-liftable runtime retention budget named in the error.

Acceptance: the runtime budget is settable from the CLI and named in the diagnostic; the snapshot parser guard remains strict and independent.

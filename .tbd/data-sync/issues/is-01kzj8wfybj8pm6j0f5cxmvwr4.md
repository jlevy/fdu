---
type: is
id: is-01kzj8wfybj8pm6j0f5cxmvwr4
title: "PR #1 review S1: Make public EntryId handles generation-safe"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:55.531Z
updated_at: 2026-08-09T03:56:45.137Z
closed_at: 2026-08-09T03:56:45.136Z
close_reason: EntryId now carries slot generation; reused slots get a new handle identity and all public id-based accessors return None for stale handles. ABA regression test and workspace suite pass.
---
PR #1 non-blocking suggestion S1. File: crates/fdu/src/index.rs. Prevent free-slot ABA by adding a generation to public handles or making raw slot IDs crate-private. Stale handles must return None rather than panic or alias a new entry.

---
type: is
id: is-01kzj8wffqtbceqnxt2718wg9q
title: "PR #1 review C5: Enforce directory-only ancestor chains"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:55.062Z
updated_at: 2026-08-09T03:54:45.689Z
closed_at: 2026-08-09T03:54:45.688Z
close_reason: Conflicting non-directory ancestors are removed and replaced by placeholder directories before child attachment; rollups and stats are tested.
---
PR #1 Cursor thread C5: https://github.com/jlevy/fdu/pull/1#discussion_r3742309837. File: crates/fdu/src/index.rs. Upserts may not attach children beneath file or symlink entries. Define and test replacement or escalation when an ancestor conflicts.

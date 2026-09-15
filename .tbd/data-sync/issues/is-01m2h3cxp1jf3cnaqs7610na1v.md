---
type: is
id: is-01m2h3cxp1jf3cnaqs7610na1v
title: "PR #56 review PR56B-STYLE-1: 124-column doc line at engine_contract.rs:1607"
kind: bug
status: closed
priority: 4
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T23:17:53.472Z
updated_at: 2026-09-15T00:12:03.404Z
closed_at: 2026-09-15T00:12:03.394Z
close_reason: "c5d27e2: reflowed the Commit::retained_cost doc comment to 100 columns"
resolution: null
duplicate_of: null
---
Delta review 5203772881 on PR #56. crates/fdu-core/src/engine_contract.rs:1607 at 8d2eb7f is 124 columns against max_width 100; rustfmt does not reflow comments. Reflow to 100.

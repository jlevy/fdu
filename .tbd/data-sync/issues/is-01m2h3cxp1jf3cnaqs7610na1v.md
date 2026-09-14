---
type: is
id: is-01m2h3cxp1jf3cnaqs7610na1v
title: "PR #56 review PR56B-STYLE-1: 124-column doc line at engine_contract.rs:1607"
kind: bug
status: open
priority: 4
version: 1
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T23:17:53.472Z
updated_at: 2026-09-14T23:17:53.472Z
---
Delta review 5203772881 on PR #56. crates/fdu-core/src/engine_contract.rs:1607 at 8d2eb7f is 124 columns against max_width 100; rustfmt does not reflow comments. Reflow to 100.

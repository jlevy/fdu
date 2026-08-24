---
type: is
id: is-01m0te8vfk0w5tp9337vkth4wy
title: Report work counters hide full-index traversal
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
  - performance
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:49:40.722Z
updated_at: 2026-08-24T17:49:52.362Z
---
PR 47 commit e47a535 adds the full Query report under IndexHandle::read, but ProjectionWork.report deliberately leaves entries_visited and dirs_visited at zero because a report may use maintained state or reaggregate. That distinction is exactly what the work contract must expose: a filtered report can walk the entire index, return few or zero rows, and claim zero visited entries, so the MetaBrowser performance ceiling cannot detect the hidden O(index) path. Fix: thread a Work sink or visit counter through query::report and every view traversal, charging actual index and directory visits to the report projection while keeping maintained reads constant. Add paired unfiltered and filtered tests with identical output bounds and a large difference in visited entries. Review follow-up FDU47-R8 at e47a535.

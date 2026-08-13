---
type: is
id: is-01kzx1ayjgknng1athmvhxp5qy
title: "Phase 2d: Lock basic metrics with tryscript and self-host checks"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1ayzr1y0jfja0et8gzybq
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:39.408Z
updated_at: 2026-08-13T07:45:39.831Z
---
Create tests/golden/fixtures/content-project and cli-content.tryscript.md with exact human/JSON/JSONL/YAML output, coverage and error cases, all newline conventions, prose, source, and binary inputs. Add make content-selfcheck over a git archive of tracked fdu HEAD files; validate multilingual presence, grouping identities, basic line partitions, coverage, and no binary text metrics. Run test-golden, content-selfcheck, and make check before optimization.

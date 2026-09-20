---
type: is
id: is-01m2y7c1v3etw9wkzz87vgm6ke
title: Add tree, paths, and long list formats across core and Python
kind: feature
status: closed
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7c6yzccahx1ntvy2wf1s1
  - type: blocks
    target: is-01m2y7cbprn6w292gjenv0twd7
  - type: blocks
    target: is-01m2yhsw73csne6aef665mmp4n
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T01:37:26.880Z
updated_at: 2026-09-20T07:12:32.216Z
started_at: 2026-09-20T01:39:25.996Z
closed_at: 2026-09-20T07:12:32.216Z
close_reason: "Implemented the tracked file/function plan above PR #96. Core/workspace tests, 168 CLI goldens, 48 Python tests, typing, docs formatting, and cross-platform lint passed. Final aggregate gate and stacked PR CI are tracked separately in fdu-arv8."
resolution: null
duplicate_of: null
---
Implement core-owned `tree`, `paths`, and `long` presentations of the shared list
report, alongside JSON, JSONL, and YAML. Tree preserves the existing directory roll-up
output, including hierarchy, columns, annotations, ordering, bounds, and omission
markers. Regular files contribute to directory totals rather than appearing as leaves.
Paths contains matching paths only.
Long contains size, actual age, and path.
Formats preserve filter semantics, directory metrics, and the request reference clock.
Flat output lists matches; tree output groups selected contents into directory roll-ups.

Keep grouped views as automatic human-readable tables.
Support mixed views in automatic text and machine formats; validate list-only format
incompatibilities in the engine before work begins.
Preserve existing tree depth and per-directory limits; flat limits apply to flat rows.
Disclose bounds without contaminating paths stdout.
Keep full’s existing bounded digest unchanged; do not add an unbounded list or a
different preview.

Expose the new view/format model through Python enums, report models, render APIs,
native adapters, and stubs.
Provide exact machine metrics, timestamps, signed age and reference instant with
explicit unknown handling, counts, kind, and ignored state.
Version changed report meanings/shapes.
Test unchanged default-output goldens and omitted/explicit tree equivalence, flat-format
membership equivalence, tree aggregate consistency, aliases, multi-view validation,
truncation/folding, machine decoding, and Python/Rust parity.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->

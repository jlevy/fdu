---
type: is
id: is-01m2yhsw73csne6aef665mmp4n
title: Expose list defaults, format aliases, and compatibility through the shared request model
kind: feature
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7c6yzccahx1ntvy2wf1s1
  - type: blocks
    target: is-01m2y7cbprn6w292gjenv0twd7
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-20T04:39:45.634Z
updated_at: 2026-09-20T05:16:03.198Z
---
Expose the accepted selection/view/format model through shared engine defaults and
validation. Metadata-only default view is list and its default format is tree.
Prove `fdu PATH`, `--view list`, `--format tree`, and their explicit combination are
equivalent to the pre-change default output, including directory-only tree rows,
columns, ordering, depth, limits, and omission notices.
Preserve analyzer-driven default views and automatic grouped tables.

Add `--format tree|paths|long` alongside text/json/jsonl/yaml.
Make `--tree` and `--long` aliases for formats.
Explicit conflicting choices fail clearly.
Incompatible view/format combinations fail before scanning; mixed views never disappear
silently. Keep the existing tree’s ordering, per-directory limits, and depth behavior;
document flat-row limits and tree expansion.
Formatting does not change filter membership or subtree metric definitions.
Use existing age grammar (`--modified-before=30d`).

Audit released `--view files`, `--view tree`, and Rust/Python public contracts; retain
needed compatibility names/presets at request boundaries without duplicate engine logic.
Explicit formats override legacy presentation defaults.
Preserve default output and document new spellings and any necessary schema changes.
Exercise one-shot, watch, opened reports, and cache-status format dispatch so adding
human formats does not bypass diagnostics or status handling.

Acceptance: unchanged default-output regression goldens, shared default-resolution
tests, CLI parsing and help tests, compatibility and invalid-combination matrix,
aliases/conflicts, early failures, and cross-surface request parity.
CLI must not invent filtering, aggregation, or format semantics.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->

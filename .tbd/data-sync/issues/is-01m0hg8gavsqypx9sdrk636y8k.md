---
type: is
id: is-01m0hg8gavsqypx9sdrk636y8k
title: "AnalysisSet: replace the profile enum with an analyzer bitset"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:16.565Z
updated_at: 2026-08-21T07:15:52.802Z
closed_at: 2026-08-21T07:15:52.801Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
Replace the five-value `AnalysisProfile` enum with a bitset over the shipped analyzer
registry. `lines` is bit 0 and is implicit whenever the set is non-empty, because any
analyzer that runs has already streamed the file.

- `is_enabled()` becomes "set is non-empty"
- `includes_code()` / `includes_words()` become real membership tests
- add `contains(other)` for the containment work in the sidecar bead
- `ContentProvenance::for_request` maps membership to analyzer ids: CONTENT_BASIC for
  any non-empty set, CODE_SLOC for `code`, TEXT_LOGICAL + MARKDOWN_PROSE for `words`
- `options_fingerprint` encodes the bitmask rather than the ordinal

Pure type change; no behavior change and no user-visible output change yet.
